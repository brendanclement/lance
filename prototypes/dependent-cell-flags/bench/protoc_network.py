#!/usr/bin/python3
"""Resolve Git's plain-file symlink representation for Lance protobuf builds."""
import os
import sys
from pathlib import Path

link = Path('protos')

def resolve(value):
    if link.is_file() and not link.is_symlink():
        target = link.read_text().strip()
        if target.rstrip('/') != '../../protos':
            raise SystemExit(f'Unexpected protobuf symlink target: {target!r}')
        root = (Path.cwd() / target).resolve()
        if value in ('./protos', 'protos'):
            return str(root)
        for prefix in ('./protos/', 'protos/'):
            if value.startswith(prefix):
                return str(root / value[len(prefix):])
    return value

args = []
for arg in sys.argv[1:]:
    if arg.startswith('-I') and len(arg) > 2:
        arg = '-I' + resolve(arg[2:])
    elif arg.startswith('--proto_path='):
        arg = '--proto_path=' + resolve(arg.split('=', 1)[1])
    else:
        arg = resolve(arg)
    args.append(arg)
os.execv('/usr/bin/protoc', ['/usr/bin/protoc', *args])
