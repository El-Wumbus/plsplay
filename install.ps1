cargo build --release

if ($LASTEXITCODE)
{
    exit 1
}

install_path="C:\Program Files\plsplay\"
mkdir -p "$install_path"
cp -v target\release\plsplay.exe "$install_path"

if ( $ENV:Path | select-string -SimpleMatch "$install_path" == "" ); then
{
    echo "'${install_path}' isn't in your PATH, add it to use plsplay."
}