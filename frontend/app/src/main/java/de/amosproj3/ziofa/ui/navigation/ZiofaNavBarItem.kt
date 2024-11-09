// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation

import androidx.compose.ui.graphics.vector.ImageVector

data class ZiofaNavBarItem(val text: String, val icon: ImageVector, val onClick: () -> Unit)
