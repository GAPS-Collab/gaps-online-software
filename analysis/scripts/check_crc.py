#! /usr/bin/env python
import libscrc

data      = open('/data0/gaps/nts/raw/1/1RB1_1686543977.blob', 'rb')
data      = [k for k in data.read()]
nwords    = 1024
head      = data[36:38]
chdata    = data[38:2*nwords+38]
trail     = data[38+2*nwords:38+2*nwords+4]
print (head)
trail_inv = [trail[2], trail[3], trail[0], trail[1]]
words = []
for k in range(len(chdata)):
   if k + 2 > len(chdata):
       break 
   if k == 0:
       pass
   elif k % 2 == 0:
       continue
   word = chdata[k:k+2]
   words.append(word)

testcrc = libscrc.crc32(int(0).to_bytes())
for word in words:
    #bigword = [word[1], word[0]]
    testcrc = libscrc.crc32(bytes(word), testcrc)

print(testcrc, int.from_bytes(trail), int.from_bytes(trail_inv))

f = open('event.dat', 'wb')
for k in data[0:18530]:
    f.write(int(k).to_bytes())
f.close()
print (data[18528:18530])

foo = open('event.dat', 'rb')
foo = [k for k in foo.read()]
for j in range(100):
    print (foo[j], data[j])

