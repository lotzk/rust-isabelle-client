use isabelle_client::{client::IsabelleClient, command::SessionBuildArgs};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut cl =
        IsabelleClient::connect("127.0.0.1", 49466, "428575ce-95cf-47c5-99ca-c1b96181b551");
    let s = serde_json::to_string("ken");
    println!("res = {:?}", s);
    let echor = cl.echo("{\"hi\": \"hallo\"}");
    println!("{:?}", echor.await);

    let build_args = SessionBuildArgs::session("HOL");
    let res = cl.session_build(build_args).await;
    println!("{:?}", res);
}
