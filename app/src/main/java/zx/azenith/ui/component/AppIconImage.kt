package zx.azenith.ui.component

import android.content.pm.PackageInfo
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import coil.compose.AsyncImage
import coil.request.ImageRequest
import androidx.compose.runtime.remember


@Composable
fun AppIconImage(
    packageInfo: PackageInfo,
    label: String,
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    
    // Gunakan remember agar proses loadIcon tidak dijalankan berulang kali saat recompose
    val iconDrawable = remember(packageInfo.packageName) {
        packageInfo.applicationInfo?.loadIcon(context.packageManager)
    }

    AsyncImage(
        model = ImageRequest.Builder(context)
            .data(iconDrawable) // Sekarang modelnya adalah Drawable, Coil mengerti ini
            .crossfade(true)
            .build(),
        contentDescription = label,
        modifier = modifier
    )
}

