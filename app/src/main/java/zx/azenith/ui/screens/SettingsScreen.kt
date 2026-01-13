package zx.azenith.ui.screens

import android.app.Activity
import android.content.Context
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.rounded.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.core.view.WindowCompat
import androidx.navigation.NavController // GANTI INI
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.core.content.edit
import zx.azenith.R
import zx.azenith.BuildConfig
import zx.azenith.ui.component.ExpressiveList
import zx.azenith.ui.component.ExpressiveListItem
import zx.azenith.ui.component.ExpressiveSwitchItem

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(navController: NavController) { // Parameter diubah
    val scrollBehavior = TopAppBarDefaults.pinnedScrollBehavior(rememberTopAppBarState())
    val context = LocalContext.current
    val view = LocalView.current
    val prefs = remember { context.getSharedPreferences("settings", Context.MODE_PRIVATE) }

    val listState = rememberLazyListState()

    // Setiap kali masuk ke screen ini, scroll ke index 0
    LaunchedEffect(Unit) {
        listState.scrollToItem(0)
    }
    
    Scaffold(
        modifier = Modifier.nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = { SettingsScreenTopAppBar(scrollBehavior) },
        containerColor = MaterialTheme.colorScheme.surface
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .padding(paddingValues)
                .verticalScroll(rememberScrollState())
        ) {
            Spacer(modifier = Modifier.height(8.dp))

            Text(
                text = "Personalization",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(horizontal = 28.dp, vertical = 8.dp)
            )

            ExpressiveList(
                modifier = Modifier.padding(horizontal = 16.dp),
                content = listOf {
                    ExpressiveListItem(
                        onClick = { navController.navigate("color_palette") }, // Route manual
                        headlineContent = { Text("Theme") },
                        supportingContent = { Text("Customize colors and accent") },
                        leadingContent = { Icon(Icons.Filled.Palette, null) },
                        trailingContent = { Icon(Icons.AutoMirrored.Filled.KeyboardArrowRight, null) }
                    )
                }
            )

            Text(
                text = "Features",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(horizontal = 28.dp, vertical = 16.dp)
            )

            ExpressiveList(
                modifier = Modifier.padding(horizontal = 16.dp),
                content = listOf(
                    {
                        var showToast by rememberSaveable { mutableStateOf(prefs.getBoolean("show_toast", true)) }
                        ExpressiveSwitchItem(
                            icon = Icons.Rounded.Notifications,
                            title = "Show Toast Notifications",
                            checked = showToast,
                            onCheckedChange = { bool ->
                                prefs.edit { putBoolean("show_toast", bool) }
                                showToast = bool
                            }
                        )
                    },
                    {
                        // Mengambil state awal dari SharedPreferences dengan key "debug_mode"
                        var debugMode by rememberSaveable { mutableStateOf(prefs.getBoolean("debug_mode", false)) }
                        
                        ExpressiveSwitchItem(
                            icon = Icons.Filled.BugReport, // Icon BugReport lebih cocok untuk Debug Mode
                            title = "Enable Debug Mode",
                            checked = debugMode,
                            onCheckedChange = { isEnabled ->
                                prefs.edit { putBoolean("debug_mode", isEnabled) }
                                debugMode = isEnabled
                            }
                        )
                    }
                )
            )

            Text(
                text = "About",
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.primary,
                modifier = Modifier.padding(horizontal = 28.dp, vertical = 16.dp)
            )

            ExpressiveList(
                modifier = Modifier.padding(horizontal = 16.dp),
                content = listOf {
                    ExpressiveListItem(
                        onClick = { },
                        headlineContent = { Text("About AZenith") },
                        supportingContent = { Text("Version ${BuildConfig.VERSION_NAME}") },
                        leadingContent = { Icon(Icons.Filled.ContactPage, null) }
                    )
                }
            )
            Spacer(modifier = Modifier.height(24.dp))
        }
    }
}

// ... Sertakan SettingsScreenTopAppBar di bawahnya ...



@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreenTopAppBar(scrollBehavior: TopAppBarScrollBehavior) {
    // TopAppBar bawaan otomatis menangani transisi warna kontainer saat scroll
    TopAppBar(
        title = {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier
                        .size(38.dp)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.surfaceVariant)
                ) {
                    Image(
                        painter = painterResource(R.drawable.avatar),
                        contentDescription = null,
                        contentScale = ContentScale.Crop,
                        modifier = Modifier.fillMaxSize()
                    )
                }
                Spacer(Modifier.width(12.dp))
                Text(
                    "Settings",
                    style = MaterialTheme.typography.titleLarge,
                    fontWeight = FontWeight.SemiBold
                )
            }
        },
        // Ini kuncinya: Warna status bar akan mengikuti warna yang diatur di sini
        colors = TopAppBarDefaults.topAppBarColors(
            containerColor = MaterialTheme.colorScheme.surface, // Warna awal
            scrolledContainerColor = MaterialTheme.colorScheme.surfaceContainer // Warna saat scroll
        ),
        scrollBehavior = scrollBehavior,
        windowInsets = WindowInsets.safeDrawing.only(WindowInsetsSides.Top + WindowInsetsSides.Horizontal)
    )
}
