pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Wayland
import ".."
import "../theme.js" as Theme

// The titles of the Alt+Tab panel, each under its window's thumbnail. The
// compositor draws the panel (src/alt-tab.tsx) but a composition draws no
// text: it tells the shell where its cells are, and draws this overlay layer
// over the panel. It takes no input: the panel's grab has the keyboard and
// the pointer.
PanelWindow {
    id: titles

    required property var modelData
    screen: modelData

    readonly property var panel: CompositorIpc.altTab
    readonly property bool shown: titles.panel !== null && titles.panel.output === titles.modelData.name

    visible: titles.shown
    color: "transparent"
    anchors.top: true
    anchors.bottom: true
    anchors.left: true
    anchors.right: true
    exclusionMode: ExclusionMode.Ignore
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.namespace: "no_blur"
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
    mask: Region {}

    Repeater {
        model: titles.shown ? titles.panel.cells : []

        delegate: Text {
            required property var modelData
            required property int index

            x: modelData.x + 6
            y: modelData.y
            width: modelData.width - 12
            height: titles.panel.titleHeight
            text: modelData.title
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            color: "#ffffff"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            font.bold: index === titles.panel.selected
            style: Text.Outline
            styleColor: "#66000000"
        }
    }
}
