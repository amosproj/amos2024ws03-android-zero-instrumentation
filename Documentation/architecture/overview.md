# Overview

## UI Layer

The UI layer contain the the screens composables and their ViewModels, as well as data transfer objects like screen states. Each screen gets their own package where the composable and ViewModel reside.

```plaintext
├── 📁 ui                  
├──── 📁 foo                        # One package per screen 
├────── 📁 composables              # Composables used by FooScreen.kt
├────── 📁 data                     # DTOs like screen state
├────── 📁 helpers                  # Helper functions
├────── 📄 FooScreen.kt             # Main composable of the screen
├────── 📄 FooViewModel.kt          # Corresponding view model
├──── 📁 ...

```
## Platform & API Layer
The platform layer abstracts communication with the backend or Android APIs and facilitates their use in the UI layer. The interfaces and DTOs used by the UI layer reside in the `api` package.
```plaintext
├── 📁 api                          # Interfaces and data classes shared with UI
├──── 📁 foo                 
├────── 📄 ConfigurationAccess.kt   # Interface for configuration access
├────── 📄 ConfigurationData.kt     # Data class provided by interface
├──── ...
├── 📁 platform                     # Platform abstractions to access Android or Backend APIs
├──── 📁 foo                 
├────── 📄 ConfigurationManager.kt  # Implementation of the interface
├──── ...
```

## Other important classes & composables
* ``ZIOFAApplication``: Main application, also contains **Koin module definitions**  
* ``MainActivity``: Exported, launched **main activity**
* ``ZiofaApp.kt``: **Main composable** + **nav graph** definition
* ``Routes``: Navigation Routes for nav graph
# Build targets & flavors
The frontend is built along with the Client SDK and supports the following build targets: 
* `mockDebug`: Debug symbols + mocked backend
* `realDebug`: Debug symbols + real backend
* `mockRelease`: R8 compiled + mocked backend
* `realRelease`: **R8 compiled + real backend** (-> this version should be used in production)

R8 compilation significantly decreases APK size & RAM usage, but can lead to runtime exceptions when classes are optimized out by mistake.
