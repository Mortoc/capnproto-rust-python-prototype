import asyncio
import socket
import capnp

PROTO_PATH = "../protos/"
greeter_capnp = capnp.load(PROTO_PATH + "greeter.capnp")


class GreeterImpl(greeter_capnp.Greeter.Server):
    def greet(self, request):
        print(f"Received request from {request.text}")
        return greeter_capnp.Response(text=f"Hello, {request.text}!")


class Server:
    async def myreader(self):
        while self.retry:
            try:
                # Must be a wait_for so we don't block on read()
                data = await asyncio.wait_for(self.reader.read(4096), timeout=0.1)
            except asyncio.TimeoutError:
                print("myreader timeout.")
                continue
            except Exception as err:
                print("Unknown myreader err: %s", err)
                return False
            await self.server.write(data)
        print("myreader done.")
        return True

    async def mywriter(self):
        while self.retry:
            try:
                # Must be a wait_for so we don't block on read()
                data = await asyncio.wait_for(self.server.read(4096), timeout=0.1)
                self.writer.write(data.tobytes())
            except asyncio.TimeoutError:
                print("mywriter timeout.")
                continue
            except Exception as err:
                print("Unknown mywriter err: %s", err)
                return False
        print("mywriter done.")
        return True

    async def myserver(self, reader, writer):
        # Start TwoPartyServer using TwoWayPipe (only requires bootstrap)
        self.server = capnp.TwoPartyServer(bootstrap=GreeterImpl())
        self.reader = reader
        self.writer = writer
        self.retry = True

        # Assemble reader and writer tasks, run in the background
        coroutines = [self.myreader(), self.mywriter()]
        tasks = asyncio.gather(*coroutines, return_exceptions=True)

        while True:
            self.server.poll_once()
            # Check to see if reader has been sent an eof (disconnect)
            if self.reader.at_eof():
                self.retry = False
                break
            await asyncio.sleep(0.01)

        # Make wait for reader/writer to finish (prevent possible resource leaks)
        await tasks


async def new_connection(reader, writer):
    print("New connection")
    server = Server()
    await server.myserver(reader, writer)


async def main():
    addr = "0.0.0.0"
    port = "51001"

    print(f"Starting server at {addr}:{port}")
    try:
        # IPv4
        server = await asyncio.start_server(
            new_connection, addr, port, family=socket.AF_INET
        )
    except Exception:
        # IPv6
        server = await asyncio.start_server(
            new_connection, addr, port, family=socket.AF_INET6
        )

    async with server:
        await server.serve_forever()


if __name__ == "__main__":
    asyncio.run(main())
