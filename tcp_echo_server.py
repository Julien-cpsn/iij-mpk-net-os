import socket

host = '192.168.179.2'
port = 5555
print(host, port)
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect((host, port))

while True:
    s.sendall(b'Hello, world')
    data = s.recv(1024)
    print('Received', repr(data))

s.close()