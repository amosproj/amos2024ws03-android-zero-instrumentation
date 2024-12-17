// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation.data

data class MenuOptionData(val title: String, val logoEmoji: String, val onClick: () -> Unit)
