use crate::greeter_capnp::greeter;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use futures::AsyncReadExt;

pub mod greeter_capnp {
    include!("../protos/greeter_capnp.rs");
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::task::LocalSet::new()
        .run_until(async move {
            let addr = "127.0.0.1:51051";
            println!("Connecting to {}", addr);
            let stream = tokio::net::TcpStream::connect(&addr).await?;
            stream.set_nodelay(true)?;
            let (reader, writer) =
                tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
            let rpc_network = Box::new(twoparty::VatNetwork::new(
                futures::io::BufReader::new(reader),
                futures::io::BufWriter::new(writer),
                rpc_twoparty_capnp::Side::Client,
                Default::default(),
            ));
            let mut rpc_system = RpcSystem::new(rpc_network, None);
            let greeter: greeter::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

            tokio::task::spawn_local(rpc_system);

            let mut request = greeter.greet_request();
            request.get().init_request().set_text("TEST");

            let reply = request.send().promise.await?;

            println!(
                "Received: {}",
                reply.get()?.get_response()?.get_text()?.to_str()?
            );
            Ok(())
        })
        .await
}
