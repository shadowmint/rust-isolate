### Design goals

- It should be possible to build an N-N peer network application (eg. chat).
- It should be possible for individuals to disconnect and the shared state is not lost.
- It should not be bound to any specific network protocol or implementation (eg. websockets)

#### More detail...

- It should be possible to spawn an agent, and have it live for an arbitrary lifespan.
- It should be possible, with no more than an *identity token* send a message to a running agent.

When an agent receives a message, it should return a communication channel to talk to it.

- Responses to messages should be sent via the channel.
- Push events should be sent via the channel.
- The channel should be strongly typed. 