"""Independent bit-vector Levenshtein, checked against a full DP matrix."""
def distance(a,b):
 if len(a)>len(b):a,b=b,a
 n=len(a)
 if not n:return len(b)
 equal={}
 for i,x in enumerate(a):equal[x]=equal.get(x,0)|(1<<i)
 mask=(1<<n)-1;high=1<<(n-1);positive=mask;negative=0;score=n
 for x in b:
  matches=equal.get(x,0);vertical=matches|negative
  horizontal=(((matches&positive)+positive)^positive)|matches
  plus=negative|~(horizontal|positive);minus=positive&horizontal
  score+=bool(plus&high)-bool(minus&high)
  plus=(plus<<1)|1;minus<<=1
  positive=(minus|~(vertical|plus))&mask;negative=(plus&vertical)&mask
 return score
