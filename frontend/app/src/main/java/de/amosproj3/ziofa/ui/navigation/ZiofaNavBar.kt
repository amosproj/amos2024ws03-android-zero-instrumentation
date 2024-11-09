// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.ui.navigation

import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue

/** Main nav bar for the app */
@Composable
fun ZiofaNavigationBar(navBarItems: List<ZiofaNavBarItem>) {
    var selectedTabIndex by remember { mutableIntStateOf(0) }
    NavigationBar {
        navBarItems.forEachIndexed { index, tarBarItem ->
            NavigationBarItem(
                selected = selectedTabIndex == index,
                icon = { Icon(tarBarItem.icon, contentDescription = tarBarItem.text) },
                onClick = {
                    selectedTabIndex = index
                    tarBarItem.onClick()
                },
                label = { Text(tarBarItem.text) },
            )
        }
    }
}
