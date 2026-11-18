"""Where the workspace is: the directory that holds upstream/pybot, data/ and runs/.

The harness lives in the rsbot repository (rsbot/harness, symlinked as ./harness in the
workspace). Scripts find the workspace by looking upward from themselves and from the current
directory for `upstream/pybot`, or take it from RSBOT_WORKSPACE.
"""
import os


def root():
    env = os.environ.get("RSBOT_WORKSPACE")
    if env:
        return os.path.abspath(env)
    for start in (os.path.dirname(os.path.abspath(__file__)), os.getcwd()):
        d = start
        while True:
            if os.path.isdir(os.path.join(d, "upstream", "pybot")):
                return d
            parent = os.path.dirname(d)
            if parent == d:
                break
            d = parent
    raise SystemExit("cannot find the workspace (a directory holding upstream/pybot); set RSBOT_WORKSPACE")
