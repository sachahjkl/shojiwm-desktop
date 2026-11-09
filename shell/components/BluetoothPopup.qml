pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Bluetooth
import ".."
import "../theme.js" as Theme

// The Bluetooth panel a click on the Bluetooth icon opens, in the style of
// the network panel: the adapter and its switch at the top, then the paired
// devices (the connected ones first, with their battery), then the devices
// in reach, found while the panel is open. A click on a device unfolds it
// with Connect or Disconnect, Pair, and Remove. The link at the bottom opens
// the Bluetooth settings (blueman-manager, whose agent asks for PINs).
PopupWindow {
    id: popup

    property Item anchorItem: null
    property bool open: false

    signal dismissed

    /// The address of the unfolded device, or "".
    property string unfolded: ""

    readonly property var adapter: Bluetooth.defaultAdapter
    readonly property bool enabled: !!popup.adapter && popup.adapter.enabled

    /// The adapter's devices: the paired ones (connected first, then by
    /// name), then the others found nearby, by name.
    readonly property var devices: {
        var all = popup.adapter && popup.adapter.devices ? popup.adapter.devices.values : [];
        var byName = function (a, b) {
            return String(a.name).localeCompare(String(b.name));
        };
        var paired = all.filter(function (device) { return device.paired; });
        paired.sort(function (a, b) {
            return (b.connected ? 1 : 0) - (a.connected ? 1 : 0) || byName(a, b);
        });
        var nearby = all.filter(function (device) { return !device.paired; });
        nearby.sort(byName);
        return paired.concat(nearby);
    }
    readonly property int pairedCount: popup.devices.filter(function (device) {
        return device.paired;
    }).length

    color: "transparent"
    visible: popup.open && popup.anchorItem !== null

    // Devices in reach show only while the panel looks for them: from when
    // it opens to when it closes, as Windows does.
    onVisibleChanged: {
        popup.unfolded = "";
        if (popup.enabled)
            popup.adapter.discovering = popup.visible;
    }

    anchor.item: popup.anchorItem
    anchor.rect: Qt.rect(0, 0,
                         popup.anchorItem ? popup.anchorItem.width : 1,
                         popup.anchorItem ? popup.anchorItem.height : 1)
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap + 2
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: 290
    implicitHeight: body.implicitHeight + footer.height + 14

    function statusOf(device) {
        if (device.pairing)
            return "Pairing…";
        if (device.state === BluetoothDeviceState.Connecting)
            return "Connecting…";
        if (device.state === BluetoothDeviceState.Disconnecting)
            return "Disconnecting…";
        if (device.connected) {
            return device.batteryAvailable
                ? "Connected · battery " + Math.round(device.battery * 100) + " %"
                : "Connected";
        }
        return device.paired ? "Paired" : "Not paired";
    }

    function iconOf(device) {
        return Quickshell.iconPath(device.icon || "bluetooth", "bluetooth");
    }

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
        radius: Math.max(0, Scheme.popupRadius - 1)
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    component Heading: Text {
        color: Scheme.menuHeading
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSizeSmall
    }

    Column {
        id: body

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 10
        spacing: 8

        // The adapter and its switch.
        Item {
            width: parent.width
            height: 34

            Image {
                id: adapterIcon
                anchors.verticalCenter: parent.verticalCenter
                width: 28
                height: 28
                sourceSize: Qt.size(56, 56)
                source: Qt.resolvedUrl("../assets/bluetooth" + (popup.enabled ? "" : "-off") + ".svg")
            }

            Column {
                anchors.left: adapterIcon.right
                anchors.leftMargin: 10
                anchors.verticalCenter: parent.verticalCenter

                Text {
                    width: 150
                    text: popup.adapter ? popup.adapter.name : "Bluetooth"
                    color: Scheme.text
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                    font.bold: true
                    elide: Text.ElideRight
                }

                Text {
                    text: !popup.enabled ? "Bluetooth is off"
                        : popup.adapter.discovering ? "Looking for devices…" : "On"
                    color: Scheme.textMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                }
            }

            Text {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                visible: !!popup.adapter
                text: popup.enabled ? "Turn off" : "Turn on"
                color: Scheme.link
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                font.underline: powerSwitch.containsMouse

                MouseArea {
                    id: powerSwitch
                    anchors.fill: parent
                    anchors.margins: -3
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        var on = !popup.enabled;
                        popup.adapter.enabled = on;
                        if (on)
                            popup.adapter.discovering = true;
                    }
                }
            }
        }

        Rectangle {
            width: parent.width
            height: 1
            color: Scheme.menuDivider
            visible: popup.enabled
        }

        Heading {
            visible: popup.enabled
            text: popup.devices.length === 0 ? "No devices found yet."
                : popup.pairedCount > 0 ? "Devices" : "Devices in reach"
        }

        ListView {
            id: list

            width: parent.width
            height: Math.min(contentHeight, 300)
            visible: popup.enabled && popup.devices.length > 0
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            spacing: 1
            model: popup.devices

            delegate: Rectangle {
                id: row

                required property var modelData
                required property int index

                readonly property var device: row.modelData
                readonly property bool unfolded: popup.unfolded === row.device.address
                /// The first device in reach, after the paired ones: a
                /// heading goes above it.
                readonly property bool firstNearby: !row.device.paired
                    && row.index === popup.pairedCount && popup.pairedCount > 0

                width: list.width
                height: (row.firstNearby ? 22 : 0)
                    + (row.unfolded ? buttons.y + buttons.height + 8 : 38)
                radius: Scheme.itemRadius
                color: "transparent"

                Heading {
                    visible: row.firstNearby
                    y: 4
                    text: "Devices in reach"
                }

                Rectangle {
                    id: card

                    y: row.firstNearby ? 22 : 0
                    width: parent.width
                    height: parent.height - y
                    radius: Scheme.itemRadius
                    color: row.unfolded ? Scheme.selection
                        : cardArea.containsMouse ? Scheme.hover : "transparent"
                    border.width: row.unfolded || cardArea.containsMouse ? 1 : 0
                    border.color: Scheme.selectionBorder

                    MouseArea {
                        id: cardArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: popup.unfolded = row.unfolded ? "" : row.device.address
                    }

                    Image {
                        id: deviceIcon
                        x: 6
                        y: 5
                        width: 28
                        height: 28
                        sourceSize: Qt.size(56, 56)
                        source: popup.iconOf(row.device)
                    }

                    Column {
                        x: deviceIcon.x + deviceIcon.width + 8
                        y: 4
                        width: parent.width - x - 8

                        Text {
                            width: parent.width
                            text: row.device.name || row.device.address
                            color: Scheme.text
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSize
                            font.bold: row.device.connected
                            elide: Text.ElideRight
                        }

                        Text {
                            text: popup.statusOf(row.device)
                            color: Scheme.textMuted
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                        }
                    }

                    Row {
                        id: buttons

                        anchors.right: parent.right
                        anchors.rightMargin: 8
                        y: 42
                        spacing: 6
                        visible: row.unfolded

                        CpButton {
                            visible: row.device.paired
                            label: "Remove"
                            onClicked: {
                                popup.unfolded = "";
                                row.device.forget();
                            }
                        }

                        CpButton {
                            visible: !row.device.paired
                            label: row.device.pairing ? "Cancel" : "Pair"
                            onClicked: {
                                if (row.device.pairing) {
                                    row.device.cancelPair();
                                } else {
                                    // Trusted, so it may connect again on its own.
                                    row.device.trusted = true;
                                    row.device.pair();
                                }
                            }
                        }

                        CpButton {
                            visible: row.device.paired
                            label: row.device.connected ? "Disconnect" : "Connect"
                            onClicked: {
                                if (row.device.connected)
                                    row.device.disconnect();
                                else
                                    row.device.connect();
                            }
                        }
                    }
                }
            }
        }

        Text {
            visible: !popup.adapter
            text: "No Bluetooth adapter."
            color: Scheme.textMuted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }
    }

    // The link to the Bluetooth settings, on the band at the bottom.
    Rectangle {
        id: footer

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 1
        height: 28
        radius: Math.max(0, Scheme.popupRadius - 1)
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
            text: "Open Bluetooth Settings"
            color: Scheme.link
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            font.underline: settingsArea.containsMouse

            MouseArea {
                id: settingsArea
                anchors.fill: parent
                anchors.margins: -4
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    Quickshell.execDetached(["blueman-manager"]);
                    popup.dismissed();
                }
            }
        }
    }
}
