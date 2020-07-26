# WinRT Image From Icon

We need to display .EXE files icons in our WinRT based Xaml container. We need to find out how this can be done.

## Entry points over the WinRT side

The XAML uses the [Image](https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.controls.image?view=winrt-19041) class to show images. This is a child of the UIElement class which is hostable in a Xaml container.

We need to use `Source` property of the `Image` class. Most examples point to an URI type of value that makes sure to load and decode the image. We will need to use a `Source` allowing us to set the source from a raw buffer of alpha enabled pixels.

The values that can go into an Image source are:

- URI (from Windows::Foundation)
- [BitmapImage](https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.media.imaging.bitmapimage?view=winrt-19041)
- [RenderTargetBitmap](https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.media.imaging.rendertargetbitmap?view=winrt-19041)
- [SoftwareBitmapSource](https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.media.imaging.softwarebitmapsource?view=winrt-19041)
- [SurfaceImageSource](https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.media.imaging.surfaceimagesource?view=winrt-19041)
- [SvgImageSource](https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.media.imaging.svgimagesource?view=winrt-19041)

## Next we need a way to make an HICON into any of the above

Candidates could be any of the 3, `BitmapImage`, `RenderTargetBitmap`, `SoftwareBitmapSource`. `BitmapImage` has a method `BitmapSource.SetSource(IRandomAccessStream)` which takes an [IRandomAccessStream](https://docs.microsoft.com/en-us/uwp/api/windows.storage.streams.randomaccessstream?view=winrt-19041) as input. Can this be use from a memory stream? Will it decode the ICO file?

### How to make BitmapImage from a memory buffer

Step 1, generate a stream tied to a memory buffer

[InMemoryRandomAccessStream](https://docs.microsoft.com/en-us/uwp/api/windows.storage.streams.inmemoryrandomaccessstream?view=winrt-19041) implements `IRandomAccessStream` which can be given to the bitmap source directly. We can generate a new `RandomAccessStream` which implements both `IOutputStream` and `IInputStream` by using the [DataWriter](https://docs.microsoft.com/en-us/uwp/api/windows.storage.streams.datawriter?view=winrt-19041) to write bytes from a memory location.

## HICON to memory HBITMAP for getting a buffer with raw pixels from an icon
```C
HDC hDC = GetDC(NULL);
HDC hMemDC = CreateCompatibleDC(hDC);
HBITMAP hMemBmp = CreateCompatibleBitmap(hDC, x, y);
HBITMAP hResultBmp = NULL;
HGDIOBJ hOrgBMP = SelectObject(hMemDC, hMemBmp);

DrawIconEx(hMemDC, 0, 0, hIcon, x, y, 0, NULL, DI_NORMAL);

hResultBmp = hMemBmp;
hMemBmp = NULL;

SelectObject(hMemDC, hOrgBMP);
DeleteDC(hMemDC);
ReleaseDC(NULL, hDC);
DestroyIcon(hIcon);
return hResultBmp;
```

# Questions to be answered
- When giving a buffer as a stream to `BitmapImage` what should be the format? Can we manipulate that using `BitmapImage` interface to set a certain decoder/format to be used from the source and then make sure we write in that format?
- Do any of the outher compatible `Image.Source` provide a better way for achieving this HICON->memory pixels->Image.Source flow? (Hint: `SoftwareBitmap`, might).
This is an interesting snippet:
```C++
  SoftwareBitmap bitmap(BitmapPixelFormat::Rgba16, 30, 30);
  bitmap.CopyFromBuffer(*stream);
```
SoftwareBitmap has a connection to the Win32 IWIC Windows API.

- How to get the pixels from a HICON to a buffer?


