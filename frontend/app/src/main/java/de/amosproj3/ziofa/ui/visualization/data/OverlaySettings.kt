// SPDX-FileCopyrightText: 2025 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.visualization.data

enum class OverlayPosition(val displayName: String) {
    TopLeft("Top Left"),
    TopRight("Top Right"),
    BottomLeft("Bottom Left"),
    BottomRight("Bottom Right"),
}

private val DEFAULT_AVAILABLE_POSITIONS =
    listOf(
        OverlayPosition.TopLeft,
        OverlayPosition.TopRight,
        OverlayPosition.BottomLeft,
        OverlayPosition.BottomRight,
    )

private val DEFAULT_OVERLAY_POSITION = OverlayPosition.BottomLeft

data class OverlaySettings(
    val selectedPosition: OverlayPosition = DEFAULT_OVERLAY_POSITION,
    val availablePositions: List<OverlayPosition> = DEFAULT_AVAILABLE_POSITIONS,
    val pctOfScreen: Float = 0.3f,
)
