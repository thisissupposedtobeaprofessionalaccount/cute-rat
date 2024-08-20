# Cute Rat
 
This is a basic remote administration tool that execute a command after parsing it from a TCP message received on a specified port. It then return the output to the sender.

i.e. you could use netcat to send some command and receive the output.

```sh
echo "ls /" | nc $HOST_IP $HOST_PORT 
```

Obviously, the binary should have the right to execute the command you want to run.
