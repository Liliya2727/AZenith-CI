package zx.azenith.ui.screens

import android.content.Context
import androidx.compose.animation.*
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.material3.pulltorefresh.PullToRefreshDefaults
import androidx.compose.material3.pulltorefresh.pullToRefresh
import androidx.compose.material3.pulltorefresh.rememberPullToRefreshState
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.TopAppBarScrollBehavior
import androidx.compose.material3.rememberTopAppBarState
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.AsyncImage
import coil.request.ImageRequest
import androidx.lifecycle.viewmodel.compose.viewModel
import zx.azenith.R
import zx.azenith.ui.viewmodel.AppInfo
import zx.azenith.ui.component.AppIconImage
import zx.azenith.ui.viewmodel.ApplistViewmodel
import androidx.activity.compose.BackHandler
import androidx.compose.ui.platform.LocalFocusManager
import zx.azenith.ui.screens.AppSettingsScreen
import androidx.navigation.NavController // GANTI INI
import androidx.navigation.NavHostController
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner

@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ApplistScreen(navController: NavController) {
    val context = LocalContext.current
    val viewModel: ApplistViewmodel = viewModel()
    val focusRequester = remember { FocusRequester() }
    val focusManager = LocalFocusManager.current
    val listState = rememberLazyListState()
    val lifecycleOwner = LocalLifecycleOwner.current // Tambahkan ini
    
    // --- TAMBAHKAN/PASTIKAN BAGIAN INI ADA ---
    val scrollBehavior = TopAppBarDefaults.pinnedScrollBehavior()
    val pullToRefreshState = rememberPullToRefreshState()
    
    val prefs = remember { context.getSharedPreferences("settings", Context.MODE_PRIVATE) }
    var showSystemApps by remember { 
        mutableStateOf(prefs.getBoolean("show_system_apps", false)) 
    }
    // ------------------------------------------

    var isSearchMode by remember { mutableStateOf(false) }
    var searchQuery by remember { mutableStateOf(TextFieldValue("")) }
    
    BackHandler(enabled = isSearchMode) {
        searchQuery = TextFieldValue("")
        isSearchMode = false
        focusManager.clearFocus()
    }

    LaunchedEffect(isSearchMode) {
        if (isSearchMode) {
            focusRequester.requestFocus()
        } else {
            searchQuery = TextFieldValue("")
            focusManager.clearFocus()
        }
    }

    LaunchedEffect(Unit) {
        if (viewModel.rawApps.isEmpty()) {
            viewModel.loadApps(context)
        }
    }

    // Di dalam ApplistScreen function:
    val filteredApps = remember(viewModel.rawApps, searchQuery.text, showSystemApps) {
        viewModel.rawApps.filter { app -> 
            val matchesSystemFilter = showSystemApps || !app.isSystem
            val matchesSearch = app.label.contains(searchQuery.text, ignoreCase = true) || 
                              app.packageName.contains(searchQuery.text, ignoreCase = true)
            
            matchesSystemFilter && matchesSearch
        }
    }

    
    // Trigger refresh setiap kali layar mendapatkan fokus kembali (RESUMED)
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_RESUME) {
                // Hanya refresh status config, jangan scan ulang APK (biar kenceng)
                if (viewModel.rawApps.isNotEmpty()) {
                    viewModel.refreshAppConfigStatus()
                }
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
        }
    }

    // Setiap kali masuk ke screen ini, scroll ke index 0
    LaunchedEffect(Unit) {
        listState.scrollToItem(0)
    }

    Scaffold(
        modifier = Modifier.nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            ApplistTopAppBar(
                scrollBehavior = scrollBehavior,
                isSearchMode = isSearchMode,
                onSearchModeChange = { 
                    isSearchMode = it 
                    if (!it) searchQuery = TextFieldValue("") 
                },
                searchQuery = searchQuery,
                onSearchChange = { searchQuery = it },
                // FIX: Tambahkan parameter yang kurang agar tidak error "No value passed"
                showSystemApps = showSystemApps,
                onToggleSystem = { newValue ->
                    showSystemApps = newValue
                    prefs.edit().putBoolean("show_system_apps", newValue).apply()
                },
                onRefresh = { viewModel.loadApps(context, forceRefresh = true) },
                focusRequester = focusRequester
            )
        }
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .pullToRefresh(
                    state = pullToRefreshState,
                    isRefreshing = viewModel.isRefreshing,
                    onRefresh = { viewModel.loadApps(context, forceRefresh = true) }
                )
        ) {
            LazyColumn(
                state = listState,
                modifier = Modifier.fillMaxSize(),
                contentPadding = PaddingValues(16.dp)
            ) {
                itemsIndexed(filteredApps, key = { _, app -> app.packageName }) { index, app ->
                    val shape = when {
                        filteredApps.size == 1 -> RoundedCornerShape(16.dp)
                        index == 0 -> RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp, bottomStart = 4.dp, bottomEnd = 4.dp)
                        index == filteredApps.lastIndex -> RoundedCornerShape(topStart = 4.dp, topEnd = 4.dp, bottomStart = 16.dp, bottomEnd = 16.dp)
                        else -> RoundedCornerShape(4.dp)
                    }
                    AppListItem(
                        app = app, 
                        shape = shape,
                        navController = navController
                    )
                    if (index < filteredApps.lastIndex) Spacer(modifier = Modifier.height(2.dp))
                }
            }

            PullToRefreshDefaults.LoadingIndicator(
                state = pullToRefreshState,
                isRefreshing = viewModel.isRefreshing,
                modifier = Modifier.align(Alignment.TopCenter)
            )
        }
    }
}


// ... Fungsi AppListItem, ApplistTopAppBar, dan LabelText tetap sama seperti kodemu ...
@Composable
fun AppListItem(
    app: AppInfo, 
    shape: RoundedCornerShape,
    navController: NavController,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = shape,
        color = MaterialTheme.colorScheme.surfaceColorAtElevation(1.dp),
        onClick = { navController.navigate("app_settings/${app.packageName}") }
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // --- GUNAKAN KOMPONEN INI, JANGAN ASYNCIMAGE MANUAL ---
            AppIconImage(
                packageInfo = app.packageInfo,
                label = app.label,
                modifier = Modifier
                    .size(48.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(MaterialTheme.colorScheme.surfaceVariant)
            )
            // -------------------------------------------------------

            Spacer(modifier = Modifier.width(16.dp))

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = app.label,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = app.packageName,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.outline,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                
                Row(modifier = Modifier.padding(top = 4.dp)) {
                    if (app.isEnabledInConfig) {
                        LabelText("ENABLED", Color(0xFF4CAF50))
                    } else {
                        LabelText("DISABLED", MaterialTheme.colorScheme.error)
                    }

                    if (app.isRecommended) {
                        LabelText("RECOMMENDED", MaterialTheme.colorScheme.primary)
                    }

                    if (app.isSystem) {
                        LabelText("SYSTEM", MaterialTheme.colorScheme.secondary)
                    }
                }
            }
        }
    }
}


@OptIn(ExperimentalMaterial3Api::class, ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun ApplistTopAppBar(
    scrollBehavior: TopAppBarScrollBehavior,
    isSearchMode: Boolean,
    onSearchModeChange: (Boolean) -> Unit,
    searchQuery: TextFieldValue,
    onSearchChange: (TextFieldValue) -> Unit,
    showSystemApps: Boolean,
    onToggleSystem: (Boolean) -> Unit,
    onRefresh: () -> Unit,
    focusRequester: FocusRequester
) {
    var menuExpanded by remember { mutableStateOf(false) }

    TopAppBar(
        scrollBehavior = scrollBehavior,
        title = {
            AnimatedContent(
                targetState = isSearchMode,
                transitionSpec = { fadeIn() togetherWith fadeOut() }
            ) { searching ->
                if (searching) {
                    TextField(
                        value = searchQuery,
                        onValueChange = onSearchChange,
                        placeholder = { Text("Search apps...") },
                        singleLine = true,
                        modifier = Modifier
                            .fillMaxWidth()
                            .focusRequester(focusRequester),
                        colors = TextFieldDefaults.colors(
                            focusedContainerColor = Color.Transparent,
                            unfocusedContainerColor = Color.Transparent,
                            focusedIndicatorColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent
                        )
                    )
                } else {
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
                            "Applist",
                            style = MaterialTheme.typography.titleLarge,
                            fontWeight = FontWeight.SemiBold
                        )
                    }
                }
            }
        },
        navigationIcon = {
            if (isSearchMode) {
                IconButton(onClick = { onSearchModeChange(false) }) {
                    Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
                }
            }
        },
        actions = {
            if (!isSearchMode) {
                IconButton(onClick = { onSearchModeChange(true) }) {
                    Icon(Icons.Default.Search, "Search")
                }
            }
            
            IconButton(onClick = { menuExpanded = true }) {
                Icon(Icons.Default.MoreVert, "Menu")
            }
            
            DropdownMenu(expanded = menuExpanded, onDismissRequest = { menuExpanded = false }) {
                DropdownMenuItem(
                    text = { Text("Refresh") },
                    onClick = { onRefresh(); menuExpanded = false },
                    leadingIcon = { Icon(Icons.Default.Refresh, null) }
                )
                DropdownMenuItem(
                    text = { Text("Show System Apps") },
                    trailingIcon = { Checkbox(showSystemApps, null) },
                    onClick = { onToggleSystem(!showSystemApps); menuExpanded = false }
                )
            }
        }
    )
}

@Composable
fun LabelText(text: String, color: Color) {
    Surface(
        color = color.copy(alpha = 0.1f),
        shape = RoundedCornerShape(4.dp),
        modifier = Modifier.padding(end = 6.dp)
    ) {
        Text(
            text = text,
            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
            fontSize = 9.sp,
            fontWeight = FontWeight.ExtraBold,
            color = color
        )
    }
}
