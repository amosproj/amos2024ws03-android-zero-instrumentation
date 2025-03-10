// SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>
//
// SPDX-License-Identifier: MIT

package de.amosproj3.ziofa.platform.processes

import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.graphics.drawable.Drawable
import de.amosproj3.ziofa.api.processes.InstalledPackageInfo

/** Provides information about the installed packages on the system and caches this information. */
class PackageInformationProvider(private val packageManager: PackageManager) {

    private val installedPackagesCache: Map<String, InstalledPackageInfo> by lazy {
        packageManager
            .getInstalledPackages(PackageManager.GET_META_DATA)
            .mapNotNull { installedPackage ->
                installedPackage.applicationInfo?.let {
                    installedPackage.packageName to retrieveInstalledPackageInfo(it)
                }
            }
            .toMap()
    }

    private fun retrieveInstalledPackageInfo(
        applicationInfo: ApplicationInfo
    ): InstalledPackageInfo {
        val displayName = packageManager.getApplicationLabel(applicationInfo).toString()
        val appIcon: Drawable = packageManager.getApplicationIcon(applicationInfo)
        return InstalledPackageInfo(displayName, appIcon)
    }

    /** Retrieve the [InstalledPackageInfo] for a given [packageName] from the cache. */
    fun getPackageInfo(packageName: String): InstalledPackageInfo? {
        return installedPackagesCache[packageName]
    }
}
