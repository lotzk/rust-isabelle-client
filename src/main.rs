use isabelle_client::{commands::SessionBuildStartArgs, server, IsabelleClient};

#[tokio::main]
async fn main() {
    env_logger::init();

    let (port, password) = server::run_server(None).unwrap();

    // Spawn a thread to wait for the server to end

    example(port, &password).await;
}

async fn example(port: u32, pass: &str) {
    let mut cl = IsabelleClient::connect(None, port, pass);

    let echor = cl.echo("{\"hi\": \"hallo\"}");
    println!("{:?}", echor.await);

    let build_args = SessionBuildStartArgs::session("HOL");
    let res = cl.session_build(&build_args).await;
    println!("{:?}", res);
    cl.shutdown().await.unwrap();
}
