import socket
import sys
import argparse
from comet.api.token import TokenManager
from comet import handlers


parser = argparse.ArgumentParser()
parser.add_argument("--token", required=True, help="Access token of the user")
parser.add_argument("--refresh-token", dest="refresh_token", required=True, help="Refresh token of the user")
parser.add_argument("--user-id", dest="user_id", help="Id of a user", required=True)

arguments, unknown_arguments = parser.parse_known_args()


HOST = 'localhost'
PORT = 9977

soc = socket.socket(socket.AF_INET, socket.SOCK_STREAM)


try:
    soc.bind((HOST, PORT))
except OSError:
    print(f'Unable to bind to {HOST}:{PORT}')
    raise


print("Listening on", HOST, PORT)
token_mgr = TokenManager(arguments)
while True:
    soc.listen(5)
    con, address = None, None
    try:
        con, address = soc.accept()
    except KeyboardInterrupt:
        soc.close()
        sys.exit(1)

    print(address[1])
    if address[0] == '127.0.0.1':
        print("Accepting connection")
        con_handler = handlers.ConnectionHandler(con, address, token_mgr)
        con_handler.handle_conection()
    else:
        con.close()
