pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Services.Pipewire

// The default audio output, for the speaker in the notification area and its
// volume panel. PipeWire gives the volume as 0 to 1; 1 is 100 %.
Singleton {
    id: root

    readonly property PwNode sink: Pipewire.defaultAudioSink
    readonly property bool ready: root.sink !== null && root.sink.ready
        && root.sink.audio !== null
    readonly property real volume: root.ready ? root.sink.audio.volume : 0
    readonly property bool muted: root.ready && root.sink.audio.muted
    /// The volume as a whole percentage.
    readonly property int percent: Math.round(root.volume * 100)
    /// The device name Windows writes under the speaker: "Speakers" when the
    /// device has no name of its own.
    readonly property string deviceName: !root.sink ? "Speakers"
        : root.sink.nickname || root.sink.description || "Speakers"

    /// 0 to 3 waves for the level, or "muted".
    readonly property string level: !root.ready || root.muted || root.percent === 0
        ? "muted"
        : root.percent < 34 ? "1" : root.percent < 67 ? "2" : "3"

    /// The program that lists every stream, as the volume mixer of Windows.
    readonly property string mixer: "pwvucontrol"

    // PipeWire sends a node's volume only to the programs that track it.
    PwObjectTracker {
        objects: root.sink ? [root.sink] : []
    }

    function setVolume(value) {
        if (!root.ready)
            return;
        root.sink.audio.volume = Math.max(0, Math.min(1, value));
        if (root.sink.audio.muted && value > 0)
            root.sink.audio.muted = false;
    }

    /// Change the volume by a number of percent.
    function step(percent) {
        root.setVolume(Math.round(root.percent + percent) / 100);
    }

    function toggleMute() {
        if (root.ready)
            root.sink.audio.muted = !root.sink.audio.muted;
    }

    function openMixer() {
        Quickshell.execDetached([root.mixer]);
    }
}
