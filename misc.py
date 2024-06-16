
file = open("english.txt", "r")
words = file.read().split("\n")
file.close()

msg = ''

for i in words:
    msg += f'"{i}", '

print(msg)
