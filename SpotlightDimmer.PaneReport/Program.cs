// SpotlightDimmer.PaneReport: forwards one pane report line to the running
// SpotlightDimmer client over the \\.\pipe\SpotlightDimmer.PaneTracker named
// pipe.
//
// Invoked by spotlight-dimmer-tmux-report.sh (running inside WSL) through
// Windows interop on every tmux hook. The shell script owns the message
// format (wire protocol v1); this exe is a dumb forwarder so protocol
// changes never require rebuilding it.
//
// Contract (mirrors the Linux helper's D-Bus call): it must be fast and it
// must never fail the tmux hook, so every failure path - client not running,
// pipe busy, bad arguments - exits 0 silently.
using System.IO.Pipes;
using System.Text;

const string PipeName = "SpotlightDimmer.PaneTracker";
const int ConnectTimeoutMs = 200;
const int MaxMessageBytes = 1024;

if (args.Length != 1 || string.IsNullOrWhiteSpace(args[0]))
    return 0;

var payload = Encoding.UTF8.GetBytes(args[0] + "\n");
if (payload.Length > MaxMessageBytes)
    return 0;

try
{
    using var pipe = new NamedPipeClientStream(".", PipeName, PipeDirection.Out);
    pipe.Connect(ConnectTimeoutMs);
    pipe.Write(payload);
    pipe.Flush();
}
catch
{
    // Silent no-op when SpotlightDimmer is not running (same behavior as the
    // Linux helper when the daemon's D-Bus name is absent).
}

return 0;
