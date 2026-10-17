"""Read capture reports and reject mixed executable identities."""
import json

def read(directory, name, optional=False):
    path = directory / name
    if optional and not path.is_file():
        return None
    return json.loads(path.read_text(encoding='utf-8'))

def checked_reports(directory):
    reflection = read(directory, 'reflection.json')
    functions = read(directory, 'functions.json', True)
    live = read(directory, 'live.json', True)
    audit = read(directory, 'profile-audit.json', True)
    for report in (functions, live, audit):
        if report and report['image'] != reflection['image']:
            raise ValueError('capture directory mixes executable identities')
    return reflection, functions, live, audit
