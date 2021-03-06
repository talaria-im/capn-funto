// Copyright (c) 2013-2015 Sandstorm Development Group, Inc. and contributors
// Licensed under the MIT License:
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use capnp::capability::Promise;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};

use crate::hello_world_capnp::{hello_world, foobar};

use futures::{AsyncReadExt, FutureExt};

struct HelloWorldImpl;

struct FoobarImpl {
    name: String,
}

impl hello_world::Server for HelloWorldImpl {
    fn say_hello(
        &mut self,
        params: hello_world::SayHelloParams,
        mut results: hello_world::SayHelloResults,
    ) -> Promise<(), ::capnp::Error> {

        let request = params.get().unwrap().get_request().unwrap();
        let name = request.get_name().unwrap();
        let message = format!("Hello, {}!", name);
        println!("server: {}", message);

        results.get().init_reply().set_message(&message);

        Promise::ok(())
    }

    fn new_foobar(
        &mut self,
        params: hello_world::NewFoobarParams,
        mut results: hello_world::NewFoobarResults,
    ) -> Promise<(), ::capnp::Error> {
        let name = params.get().unwrap().get_name().unwrap();
        let client: foobar::Client = capnp_rpc::new_client(FoobarImpl {
            name: name.into(),
        });
        results.get().set_obj(client);
        Promise::ok(())
    }
}

impl foobar::Server for FoobarImpl {
    fn who_am_i(
        &mut self,
        _params: foobar::WhoAmIParams,
        mut results: foobar::WhoAmIResults,
    ) -> Promise<(), ::capnp::Error> {
        results.get().set_reply(&self.name);
        Promise::ok(())
    }
}

impl Drop for FoobarImpl {
    fn drop(&mut self) {
        println!("goodbye {}", self.name);
    }
}

pub async fn main() -> tokio::io::DuplexStream {
    let (client, server) = tokio::io::duplex(1024);
    let hello_world_client: hello_world::Client = capnp_rpc::new_client(HelloWorldImpl);

    let (reader, writer) = tokio_util::compat::TokioAsyncReadCompatExt::compat(server).split();
    let network = twoparty::VatNetwork::new(
        reader,
        writer,
        rpc_twoparty_capnp::Side::Server,
        Default::default(),
    );
    let rpc_system = RpcSystem::new(Box::new(network), Some(hello_world_client.clone().client));
    tokio::task::spawn_local(Box::pin(rpc_system.map(|_| ())));

    client
}
