pragma ComponentBehavior: Bound

import QtQuick
import ".."

// A combo box of the Control Panel, as Windows 7 draws one: the choice in a
// field, the drop button at its right end, and below it a list with blue
// rows under the pointer. A long list scrolls. The wheel on the field steps
// the choice.
Item {
    id: combo

    /// The labels to choose from.
    property var model: []
    property int currentIndex: 0
    property bool open: false
    /// The longest the list grows before it scrolls.
    property int maxListHeight: 220

    signal activated(int index)

    implicitWidth: 220
    implicitHeight: 24
    // The open list draws over what follows.
    z: combo.open ? 10 : 0

    function choose(index) {
        combo.open = false;
        if (index >= 0 && index < combo.model.length && index !== combo.currentIndex)
            combo.activated(index);
    }

    Rectangle {
        id: field

        anchors.fill: parent
        radius: Scheme.corner(2)
        color: Scheme.field
        border.width: 1
        border.color: fieldArea.containsMouse || combo.open ? Scheme.fieldFocus : Scheme.fieldBorder

        Text {
            anchors.left: parent.left
            anchors.right: dropButton.left
            anchors.leftMargin: 6
            anchors.rightMargin: 4
            anchors.verticalCenter: parent.verticalCenter
            text: combo.model[combo.currentIndex] !== undefined ? combo.model[combo.currentIndex] : ""
            color: Scheme.text
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSize
            elide: Text.ElideRight
        }

        Rectangle {
            id: dropButton

            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            anchors.margins: 1
            width: 18
            border.width: fieldArea.containsMouse || combo.open ? 1 : 0
            border.color: Scheme.selectionBorder
            gradient: Gradient {
                GradientStop {
                    position: 0.0
                    color: combo.open ? Scheme.buttonPressedTop
                        : fieldArea.containsMouse ? Scheme.buttonHotTop : Scheme.field
                }
                GradientStop {
                    position: 1.0
                    color: combo.open ? Scheme.buttonPressedBottom
                        : fieldArea.containsMouse ? Scheme.buttonHotBottom : Scheme.field
                }
            }

            Image {
                anchors.centerIn: parent
                width: 8
                height: 5
                sourceSize: Qt.size(16, 10)
                source: "../assets/" + (Scheme.dark ? "" : "ink-") + "combo-arrow.svg"
            }
        }

        MouseArea {
            id: fieldArea
            anchors.fill: parent
            hoverEnabled: true
            onClicked: combo.open = !combo.open
            onWheel: function (event) {
                combo.choose(Math.max(0, Math.min(combo.model.length - 1,
                    combo.currentIndex + (event.angleDelta.y > 0 ? -1 : 1))));
            }
        }
    }

    Rectangle {
        id: list

        visible: combo.open
        y: combo.height
        width: combo.width
        height: Math.min(combo.maxListHeight, rows.contentHeight + 2)
        color: Scheme.field
        border.width: 1
        border.color: Scheme.fieldFocus

        ListView {
            id: rows

            anchors.fill: parent
            anchors.margins: 1
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: combo.model
            currentIndex: combo.currentIndex
            onVisibleChanged: if (visible) positionViewAtIndex(combo.currentIndex, ListView.Contain)

            delegate: Rectangle {
                id: row

                required property var modelData
                required property int index

                readonly property bool lit: rowArea.containsMouse
                    || (!listHover.hovered && row.index === combo.currentIndex)

                width: rows.width
                height: 22
                color: row.lit ? "#3399ff" : "transparent"

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: 6
                    anchors.verticalCenter: parent.verticalCenter
                    text: row.modelData
                    color: row.lit ? "#ffffff" : Scheme.text
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSize
                }

                MouseArea {
                    id: rowArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: combo.choose(row.index)
                }
            }
        }

        HoverHandler {
            id: listHover
        }
    }
}
