pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."

// The network panel a click on the network icon opens, as Windows 7 shows
// it: the connection in use and its access at the top, then the wireless
// networks in reach with their bars. A click on a network unfolds it with
// its Connect button; NetworkManager asks for a new network's key. The
// link at the bottom opens the connection editor.
PopupWindow {
    id: popup

    property Item anchorItem: null
    property bool open: false

    signal dismissed

    /// The SSID of the unfolded network, or "".
    property string unfolded: ""

    color: "transparent"
    visible: popup.open && popup.anchorItem !== null

    onVisibleChanged: {
        if (visible) {
            popup.unfolded = "";
            Network.refresh();
            if (Network.wifiEnabled)
                Network.rescan();
        }
    }

    anchor.item: popup.anchorItem
    anchor.rect: Qt.rect(0, 0,
                         popup.anchorItem ? popup.anchorItem.width : 1,
                         popup.anchorItem ? popup.anchorItem.height : 1)
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Scheme.theme.popupGap + 2
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: 290
    implicitHeight: body.implicitHeight + footer.height + 14

    Rectangle {
        anchors.fill: parent
        radius: Scheme.popupRadius
        border.width: 1
        border.color: Scheme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.popupTop }
            GradientStop { position: 1.0; color: Scheme.popupBottom }
        }
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    component Heading: Text {
        color: Scheme.menuHeading
        font.family: Scheme.theme.fontFamily
        font.pixelSize: Scheme.theme.fontSizeSmall
    }

    Column {
        id: body

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 10
        spacing: 8

        Heading { text: "Currently connected to:" }

        // The connection in use: its icon, its name, its access.
        Row {
            spacing: 10

            Image {
                anchors.verticalCenter: parent.verticalCenter
                width: 28
                height: 28
                sourceSize: Qt.size(56, 56)
                source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-")
                    + (Network.primary && Network.primary.type === "wifi"
                        ? "network-wifi-" + Network.bars : "network-wired") + ".svg")
            }

            Column {
                anchors.verticalCenter: parent.verticalCenter

                Text {
                    width: 210
                    text: Network.primary ? Network.primary.name : "Not connected"
                    color: Scheme.text
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSize
                    font.bold: true
                    elide: Text.ElideRight
                }

                Text {
                    text: !Network.primary ? "No connections are available"
                        : Network.connectivity === "full" ? "Internet access"
                        : Network.connectivity === "portal" ? "Sign-in required"
                        : Network.connectivity === "unknown" ? "Connected"
                        : "No Internet access"
                    color: Scheme.textMuted
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSizeSmall
                }
            }
        }

        Rectangle {
            width: parent.width
            height: 1
            color: Scheme.menuDivider
            visible: Network.hasWifi
        }

        // The wireless section: its switch, then the networks.
        Item {
            width: parent.width
            height: 18
            visible: Network.hasWifi

            Heading {
                anchors.verticalCenter: parent.verticalCenter
                text: "Wireless Network Connection"
            }

            Text {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                text: Network.wifiEnabled ? "Turn off" : "Turn on"
                color: Scheme.link
                font.family: Scheme.theme.fontFamily
                font.pixelSize: Scheme.theme.fontSizeSmall
                font.underline: wifiSwitch.containsMouse

                MouseArea {
                    id: wifiSwitch
                    anchors.fill: parent
                    anchors.margins: -3
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: Network.setWifi(!Network.wifiEnabled)
                }
            }
        }

        ListView {
            id: list

            width: parent.width
            height: Math.min(contentHeight, 260)
            visible: Network.hasWifi && Network.wifiEnabled
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            spacing: 1
            model: Network.networks

            delegate: Rectangle {
                id: row

                required property var modelData

                readonly property bool unfolded: popup.unfolded === row.modelData.ssid
                readonly property bool asksPassword: row.modelData.secure
                    && !row.modelData.saved && !row.modelData.active

                width: list.width
                height: row.unfolded ? unfoldedPart.y + unfoldedPart.height + 8 : 30
                radius: Scheme.itemRadius
                color: row.unfolded ? Scheme.selection
                    : rowArea.containsMouse ? Scheme.hover : "transparent"
                border.width: row.unfolded || rowArea.containsMouse ? 1 : 0
                border.color: Scheme.selectionBorder

                MouseArea {
                    id: rowArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: popup.unfolded = row.unfolded ? "" : row.modelData.ssid
                }

                Text {
                    x: 8
                    y: 8
                    width: parent.width - 60
                    text: row.modelData.ssid
                    color: Scheme.text
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSize
                    font.bold: row.modelData.active
                    elide: Text.ElideRight
                }

                Row {
                    anchors.right: parent.right
                    anchors.rightMargin: 8
                    y: 7
                    spacing: 4

                    // A secured network has a small lock, as Windows shows.
                    Text {
                        visible: row.modelData.secure
                        text: "🔒"
                        color: Scheme.textMuted
                        font.pixelSize: 10
                    }

                    Image {
                        width: 16
                        height: 16
                        sourceSize: Qt.size(32, 32)
                        source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-")
                            + "network-wifi-" + Network.barsOf(row.modelData.signal) + ".svg")
                    }
                }

                Column {
                    id: unfoldedPart

                    x: 8
                    y: 30
                    width: parent.width - 16
                    spacing: 6
                    visible: row.unfolded

                    Text {
                        text: row.modelData.active ? "Connected"
                            : row.modelData.saved ? "Saved network"
                            : row.modelData.secure ? "Secured" : "Open network"
                        color: Scheme.textMuted
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSizeSmall
                    }

                    // A new secured network: NetworkManager asks its secret
                    // agent (nm-applet) for the key, in its own window. A
                    // field here would get no keys: a popup without a grab
                    // has no keyboard.
                    Text {
                        width: parent.width
                        visible: row.asksPassword
                        text: "You will be asked for the security key."
                        color: Scheme.textMuted
                        wrapMode: Text.WordWrap
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSizeSmall
                    }

                    Text {
                        width: parent.width
                        visible: Network.error.length > 0 && row.unfolded
                        text: Network.error
                        color: "#d9302a"
                        wrapMode: Text.WordWrap
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSizeSmall
                    }

                    CpButton {
                        anchors.right: parent.right
                        label: Network.busy ? "Connecting…"
                            : row.modelData.active ? "Disconnect" : "Connect"
                        onClicked: {
                            if (row.modelData.active)
                                Network.disconnect();
                            else
                                Network.connect(row.modelData);
                        }
                    }
                }
            }
        }

        Text {
            visible: Network.hasWifi && !Network.wifiEnabled
            text: "Wireless is turned off."
            color: Scheme.textMuted
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSizeSmall
        }
    }

    // The link to the connection editor, on the band at the bottom.
    Rectangle {
        id: footer

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 1
        height: 28
        radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
        color: Scheme.dark ? "#33000000" : "#80ffffff"

        Rectangle {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 1
            color: Scheme.menuDivider
        }

        Text {
            anchors.centerIn: parent
            text: "Open Network Connections"
            color: Scheme.link
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSize
            font.underline: editorArea.containsMouse

            MouseArea {
                id: editorArea
                anchors.fill: parent
                anchors.margins: -4
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    Network.openEditor();
                    popup.dismissed();
                }
            }
        }
    }
}
