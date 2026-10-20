import random,unittest
from fast_distance import distance

def matrix(a,b):
 d=[[0]*(len(b)+1) for _ in range(len(a)+1)]
 for i in range(len(a)+1):d[i][0]=i
 for j in range(len(b)+1):d[0][j]=j
 for i,x in enumerate(a,1):
  for j,y in enumerate(b,1):d[i][j]=min(d[i-1][j]+1,d[i][j-1]+1,d[i-1][j-1]+(x!=y))
 return d[-1][-1]
class FastDistance(unittest.TestCase):
 def test_empty_and_named_edits(self):
  for a,b,n in [('', '',0),('甲','',1),('','甲乙',2),('kitten','sitting',3),('甲 乙','甲乙',1),('〳〵','〱',2),('ABAB','BABA',2),('井戶','井戸',1)]:self.assertEqual(distance(a,b),n)
 def test_varied_scalar_and_word_sequences_match_full_matrix(self):
  rng=random.Random(20261020)
  for alphabet in ['甲乙丙\n　 AＡ〱', ['甲乙','word','WORD','24','']]:
   for _ in range(128):
    a=[rng.choice(alphabet) for _ in range(rng.randrange(50))];b=[rng.choice(alphabet) for _ in range(rng.randrange(50))];self.assertEqual(distance(a,b),matrix(a,b));self.assertEqual(distance(b,a),matrix(a,b))
 def test_crosses_native_word_boundaries(self):
  for n in (63,64,65,127,128,129,1024):
   a='甲乙'*(n//2)+'甲'*(n%2);b=a[:n//2]+'丙'+a[n//2:];self.assertEqual(distance(a,b),1);self.assertEqual(distance(a,'丙'*n),n)
if __name__=='__main__':unittest.main()
