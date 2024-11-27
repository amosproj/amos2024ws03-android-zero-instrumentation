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