# WinRT Image From Icon

This document is a draft research notebook containing various leads on how a HICON can be used with the Windows UI XAML `Image` control.

Nothing outlined here has been tested to work as of now.

## Entry points over the WinRT side

The XAML uses the [Image](https://docs.microsoft.com/en-us/uwp/api/windows.ui.xaml.controls.image?view=winrt-19041) class to show images. This is a child of the UIElement class which is hostable in a Xaml container.

We need to use `Source` property of the `Image` class. Most examples point to an URI type of value that makes sure to load and decode the image. We might need to use a `Source` allowing us to set the source from a raw buffer of alpha enabled pixels.

The values that can go into an `Image`.`Source` has to be an object inheriting from `ImageSource`. Currently these can be found in the docs:

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

## Windows Imaging Component API
First we take HICON and make a `IWICBitmap`.
```C++
IWICImagingFactory::CreateBitmapFromHICON
HRESULT CreateBitmapFromHICON(
  HICON      hIcon,
  IWICBitmap **ppIBitmap
);
```
`CreateBitmapFromHICON` is already available in [winapi-rs](https://docs.rs/winapi/0.3.6/winapi/um/wincodec/struct.IWICImagingFactory.html).

Then we might be able to make the `IWICBitmap` into a WinRT `SoftwareBitmap` with:

```C++
ISoftwareBitmapNative iBmp;

HRESULT CreateFromWICBitmap(
  IWICBitmap *data,
  BOOL       forceReadOnly,
  REFIID     riid,
  LPVOID     *ppv
);
```
From MSDN seen [here](https://docs.microsoft.com/en-us/windows/win32/api/windows.graphics.imaging.interop/nf-windows-graphics-imaging-interop-isoftwarebitmapnativefactory-createfromwicbitmap).


# Image UIElement from raw memory buffer
We can also create a XAML Image control that has the `Source` property set to a `SoftwareBitmap` as this bitmap object can copy pixels from an `IBuffer`. The `IBuffer` can be created with a DataWriter. Here's some sample C++ like pseudo/guess code:
```C++
#include "pch.h"
#define i32 unsinged long;
using namespace Windows::Storage::Streams::DataWriter;
using namespace Windows::Storage::Streams::IDataWriterFactory;
using namespace Windows::Graphics::Imaging::SoftwareBitmap;
using namespace Windows::Graphics::Imaging::SoftwareBitmapSource;

void CreateBlackImage(SoftwareBitmap& outputImage) {
  int width = 0, height = 0;
  int bufferLength = width * height;
  std::vector<i32> buffer(bufferLength, 0); // 2nd param initializes all buffer
  // all 0 image is also probably transparent not only black
  // have to inspect pixel format and set alpha channel to FF for a truly black image

  // initialized through factory IDataWriterFactory
  DataWriter writer;

  // the arg to WriteBytes might be a WinRT ABI type array
  // in C++ there should be vector to WinRT array conversion operator
  // within the language bindings, for other languages that might be 
  // handled differently
  writer.WriteBytes(buffer);

  IBuffer iBuff = writer.DetachBuffer();
  outputImage.CopyFromBuffer(iBuff);
}

void SetupUI(UIEelement &rootContainer) {
  Image imgControl;
  SoftwareBitmapSource imgSrc;
  SoftwareBitmap bmp;

  CreateBlackImage(&bmp);
  imgSrc.SetBitmapAsync(&bmp)
  imgControl.set_Source = imgSrc;
}
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


