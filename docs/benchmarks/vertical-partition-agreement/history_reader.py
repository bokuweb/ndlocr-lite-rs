"""Verify experiment history or restore selected original relative paths."""
from contextlib import ExitStack
from pathlib import Path, PurePosixPath
import argparse
import gzip
import hashlib
import json
import zipfile

def safe_name(name):
    path=PurePosixPath(name)
    assert name and not path.is_absolute()and '..'not in path.parts and '\\'not in name
    assert path.as_posix()==name
    assert not(len(path.parts[0])==2 and path.parts[0][1]==':')
    return path

def read_index(packet):
    return json.loads(gzip.decompress((packet/'history-index.json.gz').read_bytes()))

def verify_history(packet):
    index=read_index(packet);logical=index['logical_files'];backers=index['backing_objects']
    for name,record in logical.items():
        safe_name(name);assert record['sha256']in backers
        assert record['bytes']==backers[record['sha256']]['bytes']
    total=0
    with ExitStack()as stack:
        archives={}
        for expected,record in backers.items():
            archive_name=record['archive'];safe_name(archive_name);safe_name(record['member'])
            if archive_name not in archives:archives[archive_name]=stack.enter_context(zipfile.ZipFile(packet/archive_name))
            archive=archives[archive_name];info=archive.getinfo(record['member']);assert info.file_size==record['bytes']
            digest=hashlib.sha256();length=0
            with archive.open(info)as stream:
                for block in iter(lambda:stream.read(1024*1024),b''):digest.update(block);length+=len(block)
            assert digest.hexdigest()==expected and length==record['bytes'],(archive_name,record['member'])
            total+=length
    return dict(logical_files=len(logical),backing_objects=len(backers),verified_unique_payload_bytes=total,new_NN_calls=0,new_model_loads=0,internal_session_run_calls=0)

def restore_history(packet,output,prefix):
    verify_history(packet)
    index=read_index(packet);selected={name:record for name,record in index['logical_files'].items()if not prefix or safe_name(name).is_relative_to(safe_name(prefix))}
    assert selected,'no matching original paths'
    output.mkdir()
    with ExitStack()as stack:
        archives={}
        for name,record in selected.items():
            backer=index['backing_objects'][record['sha256']];archive_name=backer['archive']
            if archive_name not in archives:archives[archive_name]=stack.enter_context(zipfile.ZipFile(packet/archive_name))
            data=archives[archive_name].read(backer['member']);assert hashlib.sha256(data).hexdigest()==record['sha256']
            target=output/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    return len(selected)

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--restore-prefix',help='restore only this original relative path prefix')
    parser.add_argument('--output-dir',type=Path)
    args=parser.parse_args();packet=Path(__file__).resolve().parent
    if args.restore_prefix is not None:
        assert args.output_dir is not None,'--output-dir required for restoration'
        print(json.dumps(dict(restored_files=restore_history(packet,args.output_dir.resolve(),args.restore_prefix)),indent=2))
    else:print(json.dumps(verify_history(packet),indent=2))

if __name__=='__main__':main()
