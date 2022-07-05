import socket
import sys
import argparse
import json
import os
from comet.api.token import TokenManager
from comet import handlers


def get_heroic_config_path():
    if sys.platform == 'linux':
        os_path = os.path.normpath(f"{os.getenv('XDG_CONFIG_PATH', os.path.expandvars('$HOME/.config'))}/heroic/gog_store/config.json")
        flatpak_path = os.path.expandvars("$HOME/.var/app/com.heroicgameslauncher.hgl/config/heroic/gog_store/config.json")

        if os.path.exists(os_path):
            return os_path
        return flatpak_path
    elif sys.platform == 'win32':
        return os.path.expandvars("%APPDATA%/heroic/gog_store/config.json")
    elif sys.platform == 'darwin':
        return os.path.expandvars("$HOME/.config/heroic/gog_store/config.json")

def load_heroic_config():
    heroic_config_path = get_heroic_config_path()
    file = open(heroic_config_path, 'r')
    data = file.read()
    file.close()
    json_data = json.loads(data)
    return json_data["credentials"]["access_token"], json_data["credentials"]["refresh_token"], json_data["credentials"]["user_id"] 

parser = argparse.ArgumentParser()
parser.add_argument("--token", help="Access token of the user")
parser.add_argument("--refresh-token",dest="refresh_token", help="Refresh token of the user")
parser.add_argument("--user-id", dest="user_id", help="Id of a user")
parser.add_argument("--from-heroic", dest="heroic", action="store_true")

arguments, unknown_arguments = parser.parse_known_args()


HOST = 'localhost'
PORT = 9977

soc = socket.socket(socket.AF_INET, socket.SOCK_STREAM)


try:
    soc.bind((HOST, PORT))
except OSError:
    print(f'Unable to bind to {HOST}:{PORT}')
    raise

if not arguments.token and not arguments.refresh_token and not arguments.user_id:
    token, refresh_token, user_id = load_heroic_config()
elif not arguments.token or not arguments.refresh_token or not arguments.user_id:
    print("You are missing an argument use --help")
    sys.exit()
else:
    token, refresh_token, user_id = arguments.token, arguments.refresh_token , arguments.user_id

print("Listening on", HOST, PORT)
token_mgr = TokenManager(token, refresh_token, user_id)
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
