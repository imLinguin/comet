# Handle notification pusher connection
import logging
import asyncio
import aiohttp
import random
from io import BytesIO

from comet.proto.gog.protocols import pb_pb2
from comet.proto.galaxy.protocols import webbroker_service_pb2

SORT_WEBBROKER = 2

UNKNOWN_MESSAGE = 0
AUTH_REQUEST = 1
AUTH_RESPONSE = 2
SUBSCRIBE_TOPIC_REQUEST = 3
SUBSCRIBE_TOPIC_RESPONSE = 4
MESSAGE_FROM_TOPIC = 5


class NotificationPusher:
    def __init__(self, access_token, user_id):
        self.url = "wss://notifications-pusher.gog.com/"
        self.access_token = access_token
        self.logger = logging.getLogger("notification_pusher")
        self.session = None
        self.connection = None

        self.subscribed_topics = set()

    async def setup(self):
        self.session = aiohttp.ClientSession()

        self.connection = await self.session.ws_connect(self.url)

        # Authenticate
        header_pb = pb_pb2.Header()
        header_pb.type = AUTH_REQUEST
        header_pb.sort = SORT_WEBBROKER

        request_pb = webbroker_service_pb2.AuthRequest()
        request_pb.auth_token = f"Bearer {self.access_token}"

        request_content = request_pb.SerializeToString()
        header_pb.size = len(request_content)
        header_pb.oseq = random.randint(10000, 9999999)
        header_data = header_pb.SerializeToString()

        await self.connection.send_bytes(len(header_data).to_bytes(2, "big") + header_data + request_content)

    @staticmethod
    def create_subscribe_actions(topics=None):
        if topics is None:
            topics = []

        messages = []

        for topic in topics:
            header = pb_pb2.Header()
            header.sort = SORT_WEBBROKER
            header.type = SUBSCRIBE_TOPIC_REQUEST

            data = webbroker_service_pb2.SubscribeTopicRequest()
            data.topic = topic
            header.oseq = random.randint(10000, 9999999)

            data_data = data.SerializeToString()
            header.size = len(data_data)
            header_data = header.SerializeToString()
            message = len(header_data).to_bytes(2, "big") + header_data + data_data
            messages.append(message)

        return messages

    async def handle(self, game_socket):
        self.logger.info("handling_connection:started")
        while not self.connection.closed:
            try:
                message_buf = await self.connection.receive()
            except asyncio.TimeoutError as e:
                continue

            if not message_buf.data:
                return
            message_data = BytesIO(message_buf.data)

            header_size_buf = message_data.read(2)

            header_size = int.from_bytes(header_size_buf, "big")
            header_pb = pb_pb2.Header()
            header_pb.ParseFromString(message_data.read(header_size))

            self.logger.info(f"notification_message:{header_pb.sort}|{header_pb.type}")

            if header_pb.type == AUTH_RESPONSE:
                if header_pb.Extensions[pb_pb2.Response.code] == 200:
                    self.logger.info("subscribing to chat, presence, friends")

                    # Subscribe to common topics
                    messages = self.create_subscribe_actions(["chat", "presence", "friends"])

                    for message in messages:
                        await self.connection.send_bytes(message)
                else:
                    self.logger.error(
                        f"failed to authenticate socket:{header_pb.Extensions[pb_pb2.Response.code]}")
            elif header_pb.type == SUBSCRIBE_TOPIC_RESPONSE:
                resp = webbroker_service_pb2.SubscribeTopicResponse()
                resp.ParseFromString(message_data.read(header_pb.size))
                self.logger.info(f"subscribed_to:{resp.topic}")
                self.subscribed_topics.update(resp.topic)
            elif header_pb.type == MESSAGE_FROM_TOPIC:
                # Just forward the message
                game_socket.send(message_buf.data)

            await asyncio.sleep(1)

    async def close(self):
        await self.connection.close()
        await self.session.close()
