/**
 * aislop 検査の正本（j5ik2o/ci の aislop/check-aislop.mjs）を、版を固定して
 * 取り寄せて実行する小さな起動役。正本と同じく j5ik2o/ci の aislop/bootstrap.mjs
 * が正で、各リポジトリはこのファイルを scripts/aislop-check-bootstrap.mjs へ
 * 編集せずに写して使う（レビュースレッド確認の呼び出し側ファイルと同じ扱い）。
 * 写しを直す必要が出たら、正本を直してから各リポジトリへ写し直す。
 *
 * 使い方:
 *   node scripts/aislop-check-bootstrap.mjs check   検査を実行する
 *   node scripts/aislop-check-bootstrap.mjs test    正本のテストを実行する
 *
 * 版の固定は、隣の scripts/aislop-check-pin.json で行う。
 *   { "ref": "aislop-check-v1",
 *     "files": { "check-aislop.mjs": "<sha256>", "check-aislop.test.mjs": "<sha256>" } }
 * ref は j5ik2o/ci のタグ（またはコミット）で、files はその ref における
 * aislop/ 配下のファイルの SHA-256 である。取り寄せたファイルは、必ず
 * このハッシュと照合してから使う。ハッシュが合わなければ落とす。
 * 取り寄せたファイルは node_modules/.aislop-check/<ref>/ へ置き、次回からは
 * ハッシュが合う限りそれを使う（ネットワークが無くても、キャッシュがあれば
 * 動く）。置き場所が node_modules の中なのは、aislop のスキャン対象から
 * 外れるためである。リポジトリ直下へ置くと、取り寄せた正本ファイル自体が
 * スキャンの指摘を生み、検査が落ちる。
 * キャッシュへの書き込みは、プロセスごとの一時ファイルへ書いてから rename で
 * 原子的に置く。check と test などが並行しても、書き込み途中のファイルを
 * 別の実行が読むことはない（issue #21）。
 *
 * 制限時間は内側から順に、正本の scan が600秒、この起動役全体が660秒
 * （AISLOP_CHECK_RUN_TIMEOUT_MS で変更可）、呼び出し側の期限、の順に
 * 発火する。呼び出し側の外側の期限が先に発火して起動役ごと終了しても、
 * 子プロセスが残り続けないようにするためである（issue #20）。660秒は
 * 起動役の開始からの全体予算で、正本の取得（fetch、1ファイル60秒まで）
 * の時間も含む。子プロセスには残り時間だけを渡し、残りが無ければ子を
 * 起動せずに失敗する。取得1回の制限時間は、全体予算の残り時間でも
 * 頭打ちにする（issue #25）。
 * 制限時間の超過時は、直接の子だけでなくプロセスツリーごと終了する。
 * 子プロセスは専用プロセスグループで起動し、超過時はグループ全体へ終了
 * 信号を送る（猶予の後に強制終了）。強制終了の予約は、直接の子が先に
 * 終わっても解除しない（issue #24）。この起動役自身が SIGTERM や SIGINT
 * を受けたときは、受けた信号をそのまま子のグループへ転送する。検査が
 * 自分の後始末（scan のグループの終了）をできるように、即座の強制終了
 * はしない（issue #23）。起動役の猶予（10秒）は検査の猶予（3秒）より
 * 十分長く取り、検査が scan のグループへの強制終了を終える前に、起動役
 * が検査を強制終了してしまうことがないようにする。信号を転送したら
 * 全体タイマーは解除して終了処理を引き継ぎ、転送後の信号の再送で子の
 * 後始末を中断しない。子の始末が済んでから、起動役も同じ信号で死ぬ。
 * 起動役自身が SIGKILL で死んだ場合は受け取れないが、その場合も検査は
 * 渡された残り時間の中で動き続け、scan は検査の制限時間に縛られる。
 *
 * j5ik2o/ci は公開リポジトリなので、取得に認証は要らない。
 */
import { spawn } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPTS_DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.dirname(SCRIPTS_DIR);
const PIN_PATH = path.join(SCRIPTS_DIR, "aislop-check-pin.json");
const RAW_BASE = "https://raw.githubusercontent.com/j5ik2o/ci";

/** 起動役全体の制限時間の既定（ミリ秒）。開始からの予算で、取得の時間も含む。正本の scan の制限時間（600秒）より長く、呼び出し側の期限より短く置く。 */
export const RUN_TIMEOUT_MS = 660_000;
/** 正本の取得（fetch）の制限時間（ミリ秒）。 */
export const FETCH_TIMEOUT_MS = 60_000;
/**
 * プロセスツリーへ終了信号を送ってから、強制終了まで待つ猶予（ミリ秒）。
 * 子である検査にも自前の猶予（check-aislop.mjs の TREE_KILL_GRACE_MS、
 * 3秒）があり、検査はその猶予の間に scan のグループを強制終了する。
 * 起動役の猶予が検査の猶予以下だと、検査側の強制終了が発火する前に
 * 起動役が検査を強制終了してしまい、SIGTERM を無視する scan が残る。
 * そのため起動役の猶予は、検査の猶予と、その後の検査の終了までを
 * 見込んで十分長く取る。呼び出し側の外側の期限は、この猶予も含めて
 * 全体予算より後に置く（既定では 660秒 + 10秒が内側に収まる順序）。
 */
export const TREE_KILL_GRACE_MS = 10_000;
/** 終了信号を送ったプロセスグループが消えたかを確かめる間隔（ミリ秒）。 */
const GROUP_POLL_MS = 100;

const sha256 = (buffer) => createHash("sha256").update(buffer).digest("hex");

/** 起動役全体の制限時間を読む。AISLOP_CHECK_RUN_TIMEOUT_MS の正の整数だけを上書きとして受ける。 */
export function childTimeoutMs(env = process.env) {
  const parsed = Number(env.AISLOP_CHECK_RUN_TIMEOUT_MS);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : RUN_TIMEOUT_MS;
}

/** 全体予算から経過時間を引いた、子プロセスへ渡せる残り時間（ミリ秒）。 */
export function remainingBudgetMs(budgetMs, startedAt, now = Date.now()) {
  return budgetMs - (now - startedAt);
}

/**
 * 取得1回の制限時間。固定の60秒と、全体予算の残り時間の、小さい方を使う。
 * 全体予算が60秒未満でも、取得の最中に予算が効くようにするためである
 * （issue #25）。残り時間が無い場合は 0 を返し、呼び出し側は取得を始めない。
 */
export function fetchTimeoutMs(remainingMs) {
  if (remainingMs <= 0) return 0;
  return Math.min(FETCH_TIMEOUT_MS, remainingMs);
}

/**
 * 照合済みの正本ファイルをキャッシュへ置く。プロセスごとの一時ファイルへ
 * 書いてから rename で原子的に置くので、並行する実行が書き込み途中の
 * ファイルを読むことはない。失敗したら一時ファイルを消してから投げる。
 */
export function writeVerifiedCache(cacheDir, name, buffer) {
  const cachePath = path.join(cacheDir, name);
  mkdirSync(cacheDir, { recursive: true });
  const tempPath = path.join(
    cacheDir,
    `.${name}.tmp-${process.pid}-${randomUUID()}`,
  );
  try {
    writeFileSync(tempPath, buffer);
    renameSync(tempPath, cachePath);
  } catch (error) {
    try {
      unlinkSync(tempPath);
    } catch {
      // 一時ファイルが無い・消せない場合は、元の失敗を優先して伝える。
    }
    throw error;
  }
  return cachePath;
}

async function fetchCanonicalFile(ref, name, expectedHash, cacheDir, timeoutMs) {
  const cachePath = path.join(cacheDir, name);
  if (existsSync(cachePath) && sha256(readFileSync(cachePath)) === expectedHash) {
    return cachePath;
  }
  const url = `${RAW_BASE}/${ref}/aislop/${name}`;
  let buffer;
  try {
    const response = await fetch(url, {
      signal: AbortSignal.timeout(timeoutMs),
    });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    buffer = Buffer.from(await response.arrayBuffer());
  } catch (error) {
    throw new Error(
      `正本 ${name} を ${url} から取得できませんでした: ${error.message}。キャッシュも無いか、ハッシュが合いません。ネットワークと pin の ref を確かめてください。`,
    );
  }
  const actualHash = sha256(buffer);
  if (actualHash !== expectedHash) {
    throw new Error(
      `正本 ${name} のハッシュが pin と合いません（pin: ${expectedHash}、取得: ${actualHash}）。正本が pin の ref から変わっている可能性があります。取得したファイルは使いません。`,
    );
  }
  return writeVerifiedCache(cacheDir, name, buffer);
}

/**
 * 正本のスクリプトを子プロセスで実行し、終了コードを返す。argv は
 * process.execPath の後ろに渡す引数列（検査なら [検査のパス]、テストなら
 * ["--test", テストのパス]）。子は専用プロセスグループで起動し、制限時間を
 * 超えたらグループ全体へ終了信号を送り、猶予の後に強制終了して、理由を
 * 出して 1 を返す。猶予は graceMs で変更でき、既定は TREE_KILL_GRACE_MS
 * （検査自身の猶予より十分長い値）である。強制終了の予約は、直接の子が
 * 先に終わっても解除しない
 * （issue #24）。この起動役自身が SIGTERM や SIGINT を受けたときは、受けた
 * 信号をそのまま子のグループへ転送する。検査が自分の後始末（scan の
 * グループの終了）をできるように、即座の強制終了はしない（issue #23）。
 * 転送を始めたら全体タイマーを解除して終了処理を引き継ぎ、制限時間の
 * 超過で既に送った信号も再送しない。子の始末が済んでから、起動役も
 * 同じ信号で死ぬ。起動自体に失敗した場合も
 * 1 を返す。Windows ではプロセスグループが無いため、直接の子だけを終了する。
 */
export function runPinnedCommand(
  argv,
  { cwd, env, timeoutMs, graceMs = TREE_KILL_GRACE_MS },
) {
  return new Promise((resolvePromise) => {
    const useGroup = process.platform !== "win32";
    const child = spawn(process.execPath, argv, {
      cwd,
      env,
      stdio: "inherit",
      detached: useGroup,
    });
    let timedOut = false;
    let settled = false;
    let closed = false;
    let closeStatus = null;
    let receivedSignal = null;
    const escalation = createGroupEscalation(
      child,
      useGroup,
      () => maybeFinish(),
      graceMs,
    );
    const cleanup = () => {
      clearTimeout(timer);
      escalation.stopWatch();
      process.removeListener("SIGTERM", onSigterm);
      process.removeListener("SIGINT", onSigint);
    };
    const maybeFinish = () => {
      if (settled || !closed) return;
      if (escalation.pending()) return;
      settled = true;
      cleanup();
      if (receivedSignal !== null) {
        // 子の始末が済んだので、自分も受けた信号で死ぬ。
        process.kill(process.pid, receivedSignal);
        return;
      }
      if (timedOut) {
        console.error(
          `制限時間（${Math.round(timeoutMs / 1000)}秒）を超えたため、子プロセスを終了しました。制限時間は AISLOP_CHECK_RUN_TIMEOUT_MS で変更できます。`,
        );
        resolvePromise(1);
      } else {
        resolvePromise(closeStatus ?? 1);
      }
    };
    const onSigterm = () => forward("SIGTERM");
    const onSigint = () => forward("SIGINT");
    const forward = (signal) => {
      if (receivedSignal !== null) return;
      receivedSignal = signal;
      // 2回目の信号は既定の動作で死ねるよう、受け手を外してから転送する。
      process.removeListener("SIGTERM", onSigterm);
      process.removeListener("SIGINT", onSigint);
      // 転送を始めたら全体タイマーを解除し、終了処理を escalation へ
      // 引き継ぐ。タイマーが残ると、予算の切れ目に SIGTERM が再送され、
      // 受け手を外した子が既定動作で即死して、子の後始末（scan の
      // グループの終了）が中断する。制限時間の超過で既に SIGTERM を
      // 送っている場合も、再送しない。
      clearTimeout(timer);
      if (!timedOut) {
        escalation.signalGroup(signal);
      }
      escalation.schedule();
      maybeFinish();
    };
    const timer = setTimeout(() => {
      timedOut = true;
      escalation.signalGroup("SIGTERM");
      escalation.schedule();
    }, timeoutMs);
    process.on("SIGTERM", onSigterm);
    process.on("SIGINT", onSigint);
    child.on("error", (error) => {
      console.error(`子プロセスを起動できませんでした: ${error.message}`);
      if (closed) return;
      closed = true;
      closeStatus = 1;
      maybeFinish();
    });
    child.on("close", (status) => {
      if (closed) return;
      closed = true;
      closeStatus = status;
      maybeFinish();
    });
  });
}

/**
 * 子のプロセスグループの始末を担う。グループへ信号を送る関数と、終了信号
 * の後に強制終了を予約する関数と、予約が残っているかの確認を返す。
 * 強制終了の予約は、直接の子が先に終わっても解除しない（issue #24）。
 * グループが先に消えたら、ポーリングで気づいて予約を取り消し、onSettled
 * を呼ぶ。Windows ではプロセスグループが無いため、直接の子だけを扱う。
 */
function createGroupEscalation(child, useGroup, onSettled, graceMs) {
  let escalationTimer = null;
  let groupPoll = null;
  const signalGroup = (signal) => {
    if (useGroup && child.pid) {
      try {
        process.kill(-child.pid, signal);
        return;
      } catch {
        // グループへ送れなければ、直接の子だけでも終了する。
      }
    }
    try {
      child.kill(signal);
    } catch {
      // 終了済みなら何もしない。
    }
  };
  const groupAlive = () => {
    if (!useGroup || !child.pid) return false;
    try {
      process.kill(-child.pid, 0);
      return true;
    } catch {
      return false;
    }
  };
  const stopWatch = () => {
    if (groupPoll) {
      clearInterval(groupPoll);
      groupPoll = null;
    }
  };
  const schedule = () => {
    if (escalationTimer !== null) return;
    escalationTimer = setTimeout(() => {
      escalationTimer = null;
      stopWatch();
      signalGroup("SIGKILL");
      onSettled();
    }, graceMs);
    if (useGroup) {
      groupPoll = setInterval(() => {
        if (groupAlive()) return;
        clearTimeout(escalationTimer);
        escalationTimer = null;
        stopWatch();
        onSettled();
      }, GROUP_POLL_MS);
    }
  };
  return {
    signalGroup,
    schedule,
    stopWatch,
    pending: () => escalationTimer !== null,
  };
}

async function main() {
  const startedAt = Date.now();
  const mode = process.argv[2];
  if (mode !== "check" && mode !== "test") {
    console.error(
      "使い方: node scripts/aislop-check-bootstrap.mjs check|test",
    );
    return 1;
  }
  const pin = JSON.parse(readFileSync(PIN_PATH, "utf8"));
  if (
    typeof pin.ref !== "string" ||
    typeof pin.files !== "object" ||
    pin.files === null
  ) {
    throw new Error(
      "scripts/aislop-check-pin.json の形が違います（ref と files が要ります）。",
    );
  }
  const budgetMs = childTimeoutMs(process.env);
  const cacheDir = path.join(ROOT, "node_modules", ".aislop-check", pin.ref);
  // 全体予算は取得の時間も含む。取得の前にも残り時間を確認し、取得1回の
  // 制限時間は残り時間で頭打ちにする（issue #25）。残りが無ければ、取得も
  // 子の起動もせずに失敗する。
  const fetchWithinBudget = async (name) => {
    const timeoutMs = fetchTimeoutMs(remainingBudgetMs(budgetMs, startedAt));
    if (timeoutMs <= 0) {
      console.error(
        `全体の制限時間（${Math.round(budgetMs / 1000)}秒）を使い切りました。正本の取得も子プロセスの起動も行いません。制限時間は AISLOP_CHECK_RUN_TIMEOUT_MS で変更できます。`,
      );
      return null;
    }
    return await fetchCanonicalFile(
      pin.ref,
      name,
      pin.files[name],
      cacheDir,
      timeoutMs,
    );
  };
  const checkerPath = await fetchWithinBudget("check-aislop.mjs");
  if (checkerPath === null) return 1;
  const env = {
    ...process.env,
    AISLOP_CHECK_ROOT: ROOT,
    AISLOP_CHECK_ACCEPTED: path.join(SCRIPTS_DIR, "aislop-accepted.json"),
    AISLOP_CHECK_CLI: path.join(
      ROOT,
      "node_modules",
      "aislop",
      "dist",
      "cli.js",
    ),
  };
  let argv;
  if (mode === "check") {
    argv = [checkerPath];
  } else {
    const testPath = await fetchWithinBudget("check-aislop.test.mjs");
    if (testPath === null) return 1;
    argv = ["--test", testPath];
  }
  // 子には残り時間だけを渡し、残りが無ければ子を起動せずに失敗する
  // （取得だけで予算を使い切った場合）。
  const timeoutMs = remainingBudgetMs(budgetMs, startedAt);
  if (timeoutMs <= 0) {
    console.error(
      `正本の取得だけで全体の制限時間（${Math.round(budgetMs / 1000)}秒）を使い切りました。子プロセスは起動しません。制限時間は AISLOP_CHECK_RUN_TIMEOUT_MS で変更できます。`,
    );
    return 1;
  }
  return await runPinnedCommand(argv, { cwd: ROOT, env, timeoutMs });
}

/** 実パスで比べる。シンボリックリンク経由の実行でも entry guard を一致させる。 */
function realPath(target) {
  try {
    return realpathSync(target);
  } catch {
    return path.resolve(target);
  }
}

if (
  process.argv[1] &&
  realPath(process.argv[1]) === realPath(fileURLToPath(import.meta.url))
) {
  try {
    process.exit(await main());
  } catch (error) {
    console.error(error.message);
    process.exit(1);
  }
}
