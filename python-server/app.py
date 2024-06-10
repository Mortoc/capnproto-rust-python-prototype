import asyncio
import socket
import capnp


PROTO_PATH = "../protos/"
greeter_capnp = capnp.load(PROTO_PATH + "greeter.capnp")


class GreeterImpl(greeter_capnp.Greeter.Server):
    async def greet(self, request, _context, **kwargs):
        print(f"Received request: {request.text}")
        await asyncio.sleep(1)
        return greeter_capnp.Response(text=f"Hello, {request.text}!")


async def new_connection(stream):
    print("New connection")
    await capnp.TwoPartyServer(stream, bootstrap=GreeterImpl()).on_disconnect()


async def main():
    addr = "0.0.0.0"
    port = "51051"

    print(f"Starting server at {addr}:{port}")

    async with capnp.kj_loop() as _loop:
        # IPv4
        server = await capnp.AsyncIoStream.create_server(
            new_connection, addr, port, family=socket.AF_INET
        )

        try:
            await server.serve_forever()
        except KeyboardInterrupt:
            pass
        server.close()
        await server.wait_closed()


if __name__ == "__main__":
    asyncio.run(main())
