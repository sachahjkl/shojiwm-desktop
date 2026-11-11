pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Io
import ".."

// A grid of pictures to pick from, as the Windows 7 background chooser shows
// them: the pictures in your Pictures folder and in the usual backgrounds
// folders, the built-in one first. A path can also be typed.
Item {
    id: chooser

    /// The path chosen now; "" for the built-in picture.
    property string selected: ""
    /// The picture "" stands for.
    property url builtIn: Settings.builtInWallpaper
    property string builtInLabel: "Windows 7 style (built in)"
    /// Square cells, for account pictures.
    property bool square: false
    signal chosen(string path)

    property var pictures: []

    implicitHeight: column.implicitHeight

    readonly property var folders: [
        Apps.userDir("PICTURES", "Pictures"),
        Settings.home + "/.local/share/backgrounds",
        "/run/current-system/sw/share/backgrounds",
        "/usr/share/backgrounds",
        "/usr/share/wallpapers"
    ]

    Process {
        id: finder

        running: true
        command: ["sh", "-c",
            "find \"$@\" -maxdepth 3 -type f \\( -iname '*.jpg' -o -iname '*.jpeg'"
            + " -o -iname '*.png' -o -iname '*.webp' -o -iname '*.svg' \\)"
            + " 2>/dev/null | sort | head -n 120", "find"].concat(chooser.folders)
        stdout: StdioCollector {
            onStreamFinished: {
                chooser.pictures = String(text).split("\n").filter(function (line) {
                    return line.length > 0;
                });
            }
        }
    }

    Column {
        id: column

        width: chooser.width
        spacing: 10

        Flow {
            width: parent.width
            spacing: 10

            Repeater {
                model: [""].concat(chooser.pictures)

                delegate: Item {
                    id: cell

                    required property var modelData
                    readonly property bool isSelected: chooser.selected === modelData

                    width: chooser.square ? 96 : 140
                    height: chooser.square ? 96 : 88

                    Rectangle {
                        anchors.fill: parent
                        radius: Scheme.itemRadius
                        color: cell.isSelected ? Scheme.selection
                            : cellArea.containsMouse ? Scheme.hover : "transparent"
                        border.width: cell.isSelected || cellArea.containsMouse ? 1 : 0
                        border.color: Scheme.selectionBorder
                    }

                    Image {
                        anchors.fill: parent
                        anchors.margins: 6
                        source: cell.modelData.length > 0
                            ? "file://" + cell.modelData : chooser.builtIn
                        sourceSize.width: 256
                        sourceSize.height: 256
                        fillMode: Image.PreserveAspectCrop
                        asynchronous: true
                        smooth: true
                        clip: true
                    }

                    MouseArea {
                        id: cellArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: chooser.chosen(cell.modelData)
                    }
                }
            }
        }

        Text {
            width: parent.width
            text: chooser.pictures.length === 0
                ? "No picture found in your Pictures folder. Type a path below."
                : (chooser.selected.length > 0 ? chooser.selected : chooser.builtInLabel)
            color: Scheme.textMuted
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSizeSmall
            elide: Text.ElideMiddle
        }

        // A path typed by hand.
        Row {
            spacing: 8

            Rectangle {
                width: Math.min(420, chooser.width - 110)
                height: 24
                radius: Scheme.fieldRadius
                color: Scheme.field
                border.width: 1
                border.color: pathInput.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

                TextInput {
                    id: pathInput

                    anchors.fill: parent
                    anchors.leftMargin: 6
                    anchors.rightMargin: 6
                    verticalAlignment: TextInput.AlignVCenter
                    color: Scheme.text
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSize
                    clip: true
                    selectByMouse: true
                    onAccepted: chooser.usePath()

                    Text {
                        anchors.fill: parent
                        verticalAlignment: Text.AlignVCenter
                        text: "/path/to/picture.jpg"
                        color: Scheme.placeholder
                        font: pathInput.font
                        visible: pathInput.text.length === 0
                    }
                }
            }

            CpButton {
                label: "Use this"
                onClicked: chooser.usePath()
            }
        }
    }

    function usePath() {
        var path = pathInput.text.trim().replace(/^~(?=\/)/, Settings.home);
        if (path.length === 0)
            return;
        chooser.chosen(path);
        pathInput.text = "";
    }
}
