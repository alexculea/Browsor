Add-Type -AssemblyName PresentationCore,PresentationFramework

$msg_buttons = "OK";
$msg_img = "Error"
$msg_body = 'Unsupported command, please use Add/Remove programs instead.'
[System.Windows.MessageBox]::Show($msg_body, '', $msg_buttons, $msg_img);
