<!--
SPDX-FileCopyrightText: 2024 Luca Bretting <luca.bretting@fau.de>

SPDX-License-Identifier: MIT
-->

# Frontend App for Zero Instrumentation Observability for Android

## Important Gradle tasks
### Building the APK:
`./gradlew build` <br/>
The resulting APKs for release/debug build types can then be found under `frontend/app/build/outputs/apk`.

### Installing the app:
`./gradlew installRealDebug` or `./gradlew iRD` for installing a version with a **real** backend <br/>
`./gradlew installMockDebug` or `./gradlew iMD` for installing a version with a **mocked** backend <br/>

### Generating the report for the bill of materials (BOM)
To generate a BOM of all *release runtime dependencies*, the following Gradle task can be run: <br/>
`./gradlew cyclonedxBom` <br/>
CycloneDX will generate a report in JSON format in `build/reports/bom.json`.

### Updating dependencies
To automatically update all dependencies, run
`./gradlew versionCatalogUpdate` or `./gradlew vCU`

### Format and check formatting
Format `./gradlew combinedFormat` <br/>
Check: `./gradlew ktfmtCheck`

## Installing for overlay mode
Overlay mode on AAOS requires elevated priviledges (priv-app + a runtime permission)
* `./gradlew assemble`
* `adb root`
* `adb remount`
* `adb reboot`
* `adb remount`
* `adb shell mkdir /system/priv-app/ziofa`
* `adb push app/build/outputs/apk/mock/debug/app-mock-debug.apk /system/priv-app/ziofa`
* `adb shell sync`
* `adb shell reboot`
* `adb root`
* `adb push`
* `adb shell pm grant de.amosproj3.ziofa android.permission.SYSTEM_ALERT_WINDOW`
* `adb shell pm grant --user 10 de.amosproj3.ziofa android.permission.SYSTEM_ALERT_WINDOW`

You only need to do this the first time, afterwards you can use the normal install method. 

## Troubleshooting
### The frontend crashes
Make sure the backend is running or that you are running a mocked version.
If you are running a release, check for MethodNotFoundException etc., these errors are most likely
caused by R8/ProGuard removing used classes.  
Quickfix: Use the debug build type.

### The backend and frontend crashes
Delete the local configuration to make sure it does not contain outdated entries.