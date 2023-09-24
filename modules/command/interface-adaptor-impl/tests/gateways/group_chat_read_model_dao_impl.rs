    use std::{env, thread};

    use chrono::Utc;
    use sqlx::MySqlPool;
    use testcontainers::clients::Cli;
    use testcontainers::core::WaitFor;
    use testcontainers::images::generic::GenericImage;
    use testcontainers::Container;

    use command_domain::group_chat::MemberId;
    use command_domain::group_chat::{GroupChatId, GroupChatName, MemberRole, Message};
    use command_domain::user_account::UserAccountId;
    use command_interface_adaptor_if::GroupChatReadModelUpdateDao;
    use command_interface_adaptor_impl::gateways::group_chat_read_model_dao_impl::GroupChatReadModelUpdateDaoImpl;


    fn mysql_image() -> GenericImage {
        GenericImage::new("mysql", "8.0")
            .with_exposed_port(3306)
            .with_wait_for(WaitFor::message_on_stdout("Ready for start up"))
            .with_env_var("MYSQL_ROOT_PASSWORD", "password")
            .with_env_var("MYSQL_DATABASE", "ceer")
            .with_env_var("MYSQL_USER", "ceer")
            .with_env_var("MYSQL_PASSWORD", "ceer")
    }

    mod embedded {
        use refinery::embed_migrations;

        embed_migrations!("../../../tools/refinery/migrations");
    }

    fn make_database_url_for_migration(port: u16) -> String {
        format!("mysql://root:password@localhost:{}/ceer", port)
    }

    fn make_database_url_for_application(port: u16) -> String {
        format!("mysql://ceer:ceer@localhost:{}/ceer", port)
    }

    fn refinery_migrate(port: u16) {
        let url = make_database_url_for_migration(port);
        log::debug!("url: {:?}", url);
        let opts = refinery_core::mysql::Opts::from_url(&url).unwrap();
        let mut pool_result;
        while {
            pool_result = refinery_core::mysql::Pool::new(opts.clone());
            pool_result.is_err()
        } {
            log::debug!("wait for mysql...");
            thread::sleep(std::time::Duration::from_secs(1));
        }
        let pool = pool_result.unwrap();
        let mut conn = pool.get_conn().unwrap();
        let _report = embedded::migrations::runner().run(&mut conn).unwrap();
    }

    fn init() {
        env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test() {
        let docker = Cli::docker();
        test_insert_group_chat(&docker).await;
        test_delete_group_chat(&docker).await;
        test_rename_group_chat(&docker).await;
        test_insert_member(&docker).await;
        test_delete_member(&docker).await;
        // test_post_message().await;
        // test_delete_message().await;
    }

    async fn test_insert_group_chat(docker: &Cli) {
        init();
        let mysql_node: Container<GenericImage> = docker.run(mysql_image());
        let mysql_port = mysql_node.get_host_port_ipv4(3306);

        refinery_migrate(mysql_port);

        let url = make_database_url_for_application(mysql_port);
        let pool = MySqlPool::connect(&url).await.unwrap();
        let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

        let aggregate_id = GroupChatId::new();
        let name = GroupChatName::new("test").unwrap();
        let admin_id = UserAccountId::new();

        dao
            .insert_group_chat(aggregate_id, name, admin_id, Utc::now())
            .await
            .unwrap();
    }

    async fn test_delete_group_chat(docker: &Cli) {
        init();
        let mysql_node: Container<GenericImage> = docker.run(mysql_image());
        let mysql_port = mysql_node.get_host_port_ipv4(3306);

        refinery_migrate(mysql_port);

        let url = make_database_url_for_application(mysql_port);
        let pool = MySqlPool::connect(&url).await.unwrap();
        let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

        let aggregate_id = GroupChatId::new();
        let name = GroupChatName::new("test").unwrap();
        let admin_id = UserAccountId::new();

        dao
            .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
            .await
            .unwrap();
        dao.delete_group_chat(aggregate_id).await.unwrap();
    }

    async fn test_rename_group_chat(docker: &Cli) {
        init();
        let mysql_node: Container<GenericImage> = docker.run(mysql_image());
        let mysql_port = mysql_node.get_host_port_ipv4(3306);

        refinery_migrate(mysql_port);

        let url = make_database_url_for_application(mysql_port);
        let pool = MySqlPool::connect(&url).await.unwrap();
        let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

        let aggregate_id = GroupChatId::new();
        let name = GroupChatName::new("test").unwrap();
        let admin_id = UserAccountId::new();

        dao
            .insert_group_chat(aggregate_id.clone(), name, admin_id.clone(), Utc::now())
            .await
            .unwrap();

        let name = GroupChatName::new("test-2").unwrap();
        dao.rename_group_chat(aggregate_id, name).await.unwrap();
    }

    async fn test_insert_member(docker: &Cli) {
        init();
        let mysql_node: Container<GenericImage> = docker.run(mysql_image());
        let mysql_port = mysql_node.get_host_port_ipv4(3306);

        refinery_migrate(mysql_port);

        let url = make_database_url_for_application(mysql_port);
        let pool = MySqlPool::connect(&url).await.unwrap();
        let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

        let aggregate_id = GroupChatId::new();
        let name = GroupChatName::new("test").unwrap();
        let admin_id = UserAccountId::new();

        dao
            .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
            .await
            .unwrap();

        let member_id = MemberId::new();
        let user_account_id = UserAccountId::new();
        let role = MemberRole::Member;

        dao
            .insert_member(aggregate_id, member_id, user_account_id, role, Utc::now())
            .await
            .unwrap();
    }

    async fn test_delete_member(docker: &Cli) {
        init();
        let mysql_node: Container<GenericImage> = docker.run(mysql_image());
        let mysql_port = mysql_node.get_host_port_ipv4(3306);

        refinery_migrate(mysql_port);

        let url = make_database_url_for_application(mysql_port);
        let pool = MySqlPool::connect(&url).await.unwrap();
        let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

        let aggregate_id = GroupChatId::new();
        let _seq_nr = 1;
        let name = GroupChatName::new("test").unwrap();
        let admin_id = UserAccountId::new();

        dao
            .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
            .await
            .unwrap();

        let member_id = MemberId::new();
        let user_account_id = UserAccountId::new();
        let role = MemberRole::Member;

        dao
            .insert_member(
                aggregate_id.clone(),
                member_id,
                user_account_id.clone(),
                role,
                Utc::now(),
            )
            .await
            .unwrap();

        dao.delete_member(aggregate_id, user_account_id).await.unwrap();
    }

    async fn test_post_message(docker: &Cli) {
        init();
        let mysql_node: Container<GenericImage> = docker.run(mysql_image());
        let mysql_port = mysql_node.get_host_port_ipv4(3306);

        refinery_migrate(mysql_port);

        let url = make_database_url_for_application(mysql_port);
        let pool = MySqlPool::connect(&url).await.unwrap();
        let dao = GroupChatReadModelUpdateDaoImpl::new(pool);

        let aggregate_id = GroupChatId::new();
        let _seq_nr = 1;
        let name = GroupChatName::new("test").unwrap();
        let admin_id = UserAccountId::new();

        dao
            .insert_group_chat(aggregate_id.clone(), name, admin_id, Utc::now())
            .await
            .unwrap();

        let member_id = MemberId::new();
        let user_account_id = UserAccountId::new();
        let role = MemberRole::Member;

        dao
            .insert_member(
                aggregate_id.clone(),
                member_id,
                user_account_id.clone(),
                role,
                Utc::now(),
            )
            .await
            .unwrap();

        let message = Message::new("test".to_string(), user_account_id.clone());

        dao.insert_message(aggregate_id, message, Utc::now()).await.unwrap();
    }

    async fn test_delete_message(docker: &Cli) {
        todo!() // 必須課題 難易度:中
    }
