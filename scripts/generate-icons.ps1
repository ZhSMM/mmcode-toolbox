<#
  Generate placeholder icons for Tauri.

  Run:
    powershell -ExecutionPolicy Bypass -File scripts/generate-icons.ps1

  Design:
    - 1024x1024 source PNG with radial blue gradient + white "M" letter
    - Resized to all sizes declared in tauri.conf.json
    - Single-frame ICO wrapping a 256x256 PNG (Windows Vista+ supports PNG-in-ICO)
    - .icns is intentionally NOT generated. On macOS run `npx @tauri-apps/cli icon`.
#>

$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$iconsDir = Join-Path $projectRoot 'src-tauri\icons'
New-Item -ItemType Directory -Force -Path $iconsDir | Out-Null

Add-Type -AssemblyName System.Drawing

function New-GradientBitmap {
    param(
        [int]$Size,
        [string]$Letter = 'M'
    )

    $bmp = New-Object System.Drawing.Bitmap $Size, $Size
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit

    # Radial gradient background: light blue center -> deep blue edge
    $rect = New-Object System.Drawing.Rectangle 0, 0, $Size, $Size
    $rectF = New-Object System.Drawing.RectangleF 0.0, 0.0, ([single]$Size), ([single]$Size)
    $centerColor = [System.Drawing.Color]::FromArgb(255, 96, 175, 255)
    $edgeColor   = [System.Drawing.Color]::FromArgb(255, 22, 60, 130)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddEllipse(-$Size, -$Size, $Size * 3, $Size * 3)
    $brush = New-Object System.Drawing.Drawing2D.PathGradientBrush($path)
    $brush.CenterColor = $centerColor
    $brush.SurroundColors = @($edgeColor)
    $g.FillRectangle($brush, $rect)
    $brush.Dispose()
    $path.Dispose()

    # Center "M" letter (use RectangleF + StringFormat overload explicitly)
    $fontSize = [int]($Size * 0.55)
    $font = New-Object System.Drawing.Font 'Segoe UI', $fontSize, ([System.Drawing.FontStyle]::Bold), ([System.Drawing.GraphicsUnit]::Pixel)
    $textBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::White)
    $sf = New-Object System.Drawing.StringFormat
    $sf.Alignment = [System.Drawing.StringAlignment]::Center
    $sf.LineAlignment = [System.Drawing.StringAlignment]::Center
    $g.DrawString($Letter, $font, $textBrush, $rectF, $sf)
    $font.Dispose()
    $textBrush.Dispose()
    $sf.Dispose()

    $g.Dispose()
    return $bmp
}

function Save-Png {
    param([System.Drawing.Bitmap]$Bmp, [string]$Path)
    $Bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
}

function Save-IcoFromPng {
    param([string]$PngPath, [string]$IcoPath)

    $pngBytes = [System.IO.File]::ReadAllBytes($PngPath)
    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter $ms

    # ICONDIR (6 bytes)
    $bw.Write([UInt16]0)   # reserved
    $bw.Write([UInt16]1)   # type 1 = icon
    $bw.Write([UInt16]1)   # image count

    # ICONDIRENTRY (16 bytes)
    $bw.Write([byte]0)     # width  (0 = 256)
    $bw.Write([byte]0)     # height (0 = 256)
    $bw.Write([byte]0)     # color count
    $bw.Write([byte]0)     # reserved
    $bw.Write([UInt16]1)   # color planes
    $bw.Write([UInt16]32)  # bits per pixel
    $bw.Write([UInt32]$pngBytes.Length)  # image data size
    $bw.Write([UInt32]22)  # offset to image data (6 + 16)

    # image data (raw PNG bytes)
    $bw.Write($pngBytes)

    $bw.Flush()
    [System.IO.File]::WriteAllBytes($IcoPath, $ms.ToArray())
    $bw.Dispose()
    $ms.Dispose()
}

# ---- Generate PNGs ----
$sources = @{
    '32x32.png'        = 32
    '128x128.png'      = 128
    '128x128@2x.png'   = 256
    'icon.png'         = 512
}

foreach ($name in $sources.Keys) {
    $size = $sources[$name]
    $bmp = New-GradientBitmap -Size $size
    $out = Join-Path $iconsDir $name
    Save-Png -Bmp $bmp -Path $out
    $bmp.Dispose()
    Write-Host "  + $name ($size x $size)"
}

# ---- Generate ICO from 256x256 PNG ----
$icoSrc = Join-Path $iconsDir '128x128@2x.png'
$icoDst = Join-Path $iconsDir 'icon.ico'
Save-IcoFromPng -PngPath $icoSrc -IcoPath $icoDst
Write-Host "  + icon.ico (256x256 PNG-in-ICO)"

Write-Host ""
Write-Host "All icons generated in: $iconsDir"
Write-Host "Edit New-GradientBitmap's Letter/colors and re-run to customize."