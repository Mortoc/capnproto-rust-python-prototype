use crate::greeter_capnp::*;
use futures::io::{BufReader, BufWriter};
use futures::{AsyncReadExt, TryFutureExt};

use capnp::capability::Promise;
use capnp_rpc::{pry, rpc_twoparty_capnp, twoparty, RpcSystem};

pub mod greeter_capnp {
    include!("../protos/greeter_capnp.rs");
}

struct GreeterImpl;
impl greeter::Server for GreeterImpl {
    fn greet(
        &mut self,
        params: greeter::GreetParams,
        mut results: greeter::GreetResults,
    ) -> Promise<(), capnp::Error> {
        let request = pry!(pry!(params.get()).get_request());
        let name = pry!(pry!(request.get_text()).to_str());
        let message = format!("Hello, {name}!");
        results.get().init_response().set_text(message);

        Promise::ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::task::LocalSet::new()
        .run_until(async move {
            let addr = "0.0.0.0:50051";
            let listener = tokio::net::TcpListener::bind(addr).await?;
            println!("Started server on: {addr}");

            let greeter: greeter::Client = capnp_rpc::new_client(GreeterImpl);

            loop {
                let (stream, _) = listener.accept().await?;
                stream.set_nodelay(true)?;
                let (reader, writer) =
                    tokio_util::compat::TokioAsyncReadCompatExt::compat(stream).split();
                let network = twoparty::VatNetwork::new(
                    BufReader::new(reader),
                    BufWriter::new(writer),
                    rpc_twoparty_capnp::Side::Server,
                    Default::default(),
                );

                let rpc_system = RpcSystem::new(Box::new(network), Some(greeter.clone().client));
                tokio::task::spawn_local(rpc_system.map_err(|e| println!("error: {e:?}")));
            }
        })
        .await
}
