@0xb33fcacab100dac3;

struct Request {
  text @0 :Text;
}

struct Response {
  text @0 :Text;
}

interface Greeter {
  greet @0 (request :Request) -> (response :Response);
}