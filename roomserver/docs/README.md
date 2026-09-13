# {{ product_name }} RoomServer

The {{ product_name }} RoomServer runs meeting rooms.
Before a user can connect to the RoomServer, the Controller needs to create a meeting room on the RoomServer.
Users can request an access token from the Controller and connect to the RoomServer using that token.
A room is created when the first token is requested and torn down after the last user has left the room and a timeout has passed.
During the meeting, the RoomServer executes signaling modules.
The functionality of the RoomServer is split up into multiple signaling modules.
Each module is responsible for some part of a meeting.
There is a module that provides chat functionality, one for managing LiveKit and many more.

## Content

- [Admin documentation](admin/README.md)
