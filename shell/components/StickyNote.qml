import QtQuick
import ".."
import "../theme.js" as Theme

// One sticky note on the desktop: a pale yellow sheet with a band at the
// top. Drag the band to move it, its corner to size it. The band's + adds
// another note beside it and its × removes this one. The text is kept as
// you type.
Item {
    id: note

    required property var modelData
    /// The note to give the keyboard to once it shows (a new one).
    property bool focusOnShow: false

    x: note.modelData.x
    y: note.modelData.y
    width: note.modelData.width
    height: note.modelData.height

    Component.onCompleted: {
        if (note.focusOnShow)
            editor.forceActiveFocus();
    }

    // A soft shadow under the sheet.
    Rectangle {
        anchors.fill: sheet
        anchors.topMargin: 3
        anchors.leftMargin: 3
        anchors.rightMargin: -3
        anchors.bottomMargin: -4
        radius: Scheme.corner(2)
        color: "#40000000"
    }

    Rectangle {
        id: sheet

        anchors.fill: parent
        radius: Scheme.corner(1)
        border.width: 1
        border.color: "#b8a24a"
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#fdf8b4" }
            GradientStop { position: 1.0; color: "#f8ef8e" }
        }
    }

    // The band: a slightly darker strip, the handle to move the note.
    Rectangle {
        id: band

        x: 1
        y: 1
        width: parent.width - 2
        height: 22
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#f8f09a" }
            GradientStop { position: 1.0; color: "#efe27a" }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: pressed ? Qt.ClosedHandCursor : Qt.SizeAllCursor
            property point grab
            onPressed: function (event) { grab = Qt.point(event.x, event.y); }
            onPositionChanged: function (event) {
                if (!pressed)
                    return;
                var at = mapToItem(note.parent, event.x, event.y);
                note.x = Math.max(0, Math.min(note.parent.width - note.width, at.x - grab.x));
                note.y = Math.max(0, Math.min(note.parent.height - note.height, at.y - grab.y));
            }
            onReleased: StickyNotes.update(note.modelData.id, { x: Math.round(note.x), y: Math.round(note.y) })
        }

        Text {
            anchors.left: parent.left
            anchors.leftMargin: 6
            anchors.verticalCenter: parent.verticalCenter
            text: "+"
            color: plusArea.containsMouse ? "#2a2200" : "#7a6a20"
            font.pixelSize: 17

            MouseArea {
                id: plusArea
                anchors.fill: parent
                anchors.margins: -4
                hoverEnabled: true
                onClicked: StickyNotes.add(note.modelData.screen,
                    Math.min(note.parent.width - 220, note.x + 24), note.y + 24)
            }
        }

        Text {
            anchors.right: parent.right
            anchors.rightMargin: 7
            anchors.verticalCenter: parent.verticalCenter
            text: "×"
            color: crossArea.containsMouse ? "#c42b1c" : "#7a6a20"
            font.pixelSize: 17

            MouseArea {
                id: crossArea
                anchors.fill: parent
                anchors.margins: -4
                hoverEnabled: true
                onClicked: StickyNotes.remove(note.modelData.id)
            }
        }
    }

    Flickable {
        id: flick

        x: 8
        y: band.height + 4
        width: parent.width - 16
        height: parent.height - band.height - 14
        contentHeight: editor.height
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        TextEdit {
            id: editor

            width: flick.width
            // At least the whole sheet, so a click anywhere on it types.
            height: Math.max(implicitHeight, flick.height)
            text: note.modelData.text
            color: "#2a2200"
            wrapMode: TextEdit.Wrap
            selectByMouse: true
            font.family: "Segoe Print, Comic Neue, " + Theme.fontFamily
            font.pixelSize: 14
            onTextChanged: saveLater.restart()
            onActiveFocusChanged: {
                if (activeFocus)
                    StickyNotes.editing = note.modelData.id;
            }
            Keys.onEscapePressed: {
                editor.focus = false;
                StickyNotes.editing = "";
            }
            onCursorRectangleChanged: {
                if (cursorRectangle.y + cursorRectangle.height > flick.contentY + flick.height)
                    flick.contentY = cursorRectangle.y + cursorRectangle.height - flick.height;
            }
        }

    }

    // Typing is kept a moment after the last key.
    Timer {
        id: saveLater
        interval: 600
        onTriggered: {
            if (editor.text !== note.modelData.text)
                StickyNotes.update(note.modelData.id, { text: editor.text });
        }
    }

    // The corner to size the note.
    MouseArea {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: 14
        height: 14
        cursorShape: Qt.SizeFDiagCursor
        property point grab
        onPressed: function (event) { grab = Qt.point(event.x, event.y); }
        onPositionChanged: function (event) {
            if (!pressed)
                return;
            note.width = Math.max(140, note.width + event.x - grab.x);
            note.height = Math.max(100, note.height + event.y - grab.y);
        }
        onReleased: StickyNotes.update(note.modelData.id,
            { width: Math.round(note.width), height: Math.round(note.height) })

        Text {
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 1
            text: "◢"
            color: "#c9b85a"
            font.pixelSize: 9
        }
    }
}
