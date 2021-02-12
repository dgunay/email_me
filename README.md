# Email Me

This app literally just emails me. 

More accurately, it uses AWS SNS to send me
an email via a subscription to a certain topic. 

I intend to use it for hobby projects that
I want to send me emails on limited resources (such as Raspberry Pi Zero W's).

## How to use it

Build the docker image. You may want to tag it.

```sh
docker build .
```

```
Usage: email_me <message>

Options:
    -r, --region        AWS region of the SNS (defaults to us-east-2)
    -t, --topic-arn     Topic ARN
    -s, --subject       Subject
    -e, --server        Serve requests continuously
```

You need to have an SNS topic set up beforehand. Provide IAM service account
credentials via [the environment variables][rusoto-auth] `AWS_ACCESS_KEY_ID`
and `AWS_SECRET_ACCESS_KEY`. You should also make sure to subscribe to your
topic or nothing will happen.

[rusoto-auth]: https://rusoto.github.io/rusoto/rusoto_credential/struct.EnvironmentProvider.html

It can be invoked either one time as a command line utility, or it can serve 
requests with JSON payloads that 
look like:

```json
{
    "message" : "required message body",
    "subject" : "optional subject"
}
```


## Technical details

The service primarily relies on `rusoto`, `tokio`, and `hyper`. Unforunately,
the single-use invocation has to use the `tokio` runtime due to `rusoto`'s 
dependence on `tokio` it.

It is packaged into a small, statically-linked binary and docker container using
[`rust-musl-builder`](https://github.com/emk/rust-musl-builder). It weighs
about 17MB.

Error handling is somewhat sloppy using `anyhow` and mapping whatever bubbles
up into 500 responses. Could definitely be improved with a defined set of
errors.