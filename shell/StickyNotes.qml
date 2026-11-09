pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

// The sticky notes of the desktop, as Windows 7's Sticky Notes: each has
// its screen, its place, its size and its text. They are kept in
// $XDG_STATE_HOME/shojiwm-win7/notes.json.
Singleton {
    id: root

    /// { id, screen, x, y, width, height, text }.
    property var notes: []
    property bool loaded: false
    /// The id of the note added last, which takes the keyboard.
    property string lastAdded: ""
    /// The id of the note being typed in, or "". The desktop layer holds
    /// the keyboard meanwhile: a click on it does not give it the keyboard.
    property string editing: ""
    readonly property string stateDir: (Quickshell.env("XDG_STATE_HOME")
        || (Quickshell.env("HOME") + "/.local/state")) + "/shojiwm-win7"

    function save() {
        if (root.loaded)
            store.setText(JSON.stringify(root.notes) + "\n");
    }

    /// A new empty note at a point of a screen; returns its id.
    function add(screen, x, y) {
        var id = Date.now().toString(36) + Math.floor(Math.random() * 1000).toString(36);
        root.notes = root.notes.concat([{
            id: id, screen: screen, x: Math.round(x), y: Math.round(y),
            width: 220, height: 200, text: ""
        }]);
        root.lastAdded = id;
        root.editing = id;
        root.save();
        return id;
    }

    /// Change some fields of a note. The note is changed in place, not the
    /// list: a new list would rebuild every note on the desktop, and the
    /// one being typed in would lose its cursor.
    function update(id, fields) {
        for (var i = 0; i < root.notes.length; i++) {
            if (root.notes[i].id === id)
                Object.assign(root.notes[i], fields);
        }
        root.save();
    }

    function remove(id) {
        root.notes = root.notes.filter(function (note) { return note.id !== id; });
        root.save();
    }

    Process {
        command: ["mkdir", "-p", root.stateDir]
        running: true
        onRunningChanged: {
            if (!running)
                store.path = root.stateDir + "/notes.json";
        }
    }

    FileView {
        id: store

        printErrors: false
        onLoaded: {
            if (root.loaded)
                return;
            try {
                var saved = JSON.parse(store.text());
                if (Array.isArray(saved))
                    root.notes = saved.concat(root.notes);
            } catch (error) {
            }
            root.loaded = true;
        }
        onLoadFailed: root.loaded = true
    }
}
