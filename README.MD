# Cute Rat

![cute big rat](https://github.com/user-attachments/assets/713db525-79a2-459d-b71f-e13d04efeac4)

## What is it ?
 
This is a basic remote administration tool that works with a server (rat king) from which it receive command and send the output back.

Obviously, the binary should have the right to execute the command you want to run.

## Customize your rat

There some few options you can customize in the main.rs file such as rat king's IP and port.

## Build a rat

To build this project, you can use cargo : 

```sh
cargo build --release
```

and get the build inside the usual Rust build folder ./target/release/ .

## Use the rat

Find a way to launch this on the targeted machine and use rat king to send commands.

You can also use netcat to send commands one by one : 
```sh
echo "ls -la /" | nc $CLIENT_IP $CLIENT_PORT
```
