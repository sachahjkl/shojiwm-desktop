import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The network icon of the notification area, as Windows 7 draws it: a screen
// for a wired connection, bars for a wireless one, a yellow mark when the
// connection has no Internet, and a red mark when there is none. A click
// opens the network panel.
Item {
    id: icon

    /// The network panel is open, so the icon stays lit.
    property bool active: false
    readonly property bool hovered: area.containsMouse
    signal clicked

    readonly property string glyph: Network.primary && Network.primary.type === "wifi"
        ? "network-wifi-" + Network.bars
        : Network.primary || !Network.hasWifi ? "network-wired" : "network-wifi-0"
    readonly property string badge: !Network.primary ? "error"
        : Network.connectivity === "full" || Network.connectivity === "unknown" ? "" : "warning"

    implicitWidth: Theme.trayIconSize + 8
    implicitHeight: Theme.taskbarHeight - Theme.taskbarPadding * 2

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 2
        anchors.bottomMargin: 2
        radius: Scheme.itemRadius
        color: "#ffffff"
        opacity: icon.active ? 0.22 : area.containsMouse ? 0.14 : 0
        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Theme.fade) }
        }
    }

    Image {
        id: picture
        anchors.centerIn: parent
        width: Theme.trayIconSize + 2
        height: Theme.trayIconSize + 2
        sourceSize: Qt.size(width * 2, height * 2)
        source: Qt.resolvedUrl("../assets/" + icon.glyph + ".svg")
    }

    Image {
        anchors.right: picture.right
        anchors.bottom: picture.bottom
        anchors.rightMargin: -2
        anchors.bottomMargin: -1
        width: 10
        height: 10
        sourceSize: Qt.size(20, 20)
        visible: icon.badge.length > 0
        source: icon.badge.length > 0
            ? Qt.resolvedUrl("../assets/network-badge-" + icon.badge + ".svg") : ""
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: icon.clicked()
    }

    // The tooltip of Windows 7: the network and its access.
    PopupWindow {
        anchor.item: icon
        anchor.rect: Qt.rect(0, 0, icon.width, icon.height)
        anchor.edges: Edges.Top
        anchor.gravity: Edges.Top
        anchor.margins.bottom: Theme.popupGap + 2
        visible: tipTimer.shown && area.containsMouse && !icon.active
        color: "transparent"
        implicitWidth: tip.implicitWidth + 14
        implicitHeight: tip.implicitHeight + 8

        Rectangle {
            anchors.fill: parent
            radius: Scheme.itemRadius
            border.width: 1
            border.color: Scheme.popupOutline
            gradient: Gradient {
                GradientStop { position: 0.0; color: Scheme.popupTop }
                GradientStop { position: 1.0; color: Scheme.popupBottom }
            }
        }

        Text {
            id: tip
            anchors.centerIn: parent
            text: Network.primary
                ? Network.primary.name + "\n" + (Network.connectivity === "full"
                    ? "Internet access" : "No Internet access")
                : "Not connected"
            color: Scheme.text
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }
    }

    Timer {
        id: tipTimer
        property bool shown: false
        interval: 600
        running: area.containsMouse
        onRunningChanged: if (!running) shown = false
        onTriggered: shown = true
    }
}
