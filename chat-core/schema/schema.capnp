@0xf78d017b948afd48;

struct Entity {
    timestamp @0 :Int64;
    kind :union {
        handshake @1 :Handshake;
        registration @2 :Registration;
        authentication @3 :Authentication;
        message @4 :Message;
    }
}

struct Handshake { pubKey @0 :Text; }

struct Registration {
    struct Request {
        username @0 :Text;
        password @1 :Text;
    }
    struct Response {
        enum Status {
            success @0;
            userExists @1;
        }
        status @0 :Status;
    }
    kind :union {
        request @0 :Request;
        response @1 :Response;
    }
}

struct Authentication {
    struct Request {
        username @0 :Text;
        password @1 :Text;
    }
    struct Response {
        enum Status {
            success @0;
            userDoesNotExist @1;
            wrongPassword @2;
        }
        status @0 :Status;
    }
    kind :union {
        request @0 :Request;
        response @1 :Response;
    }
}

struct Message {
    sender @0 :Text;
    text @1 :Text;
}
