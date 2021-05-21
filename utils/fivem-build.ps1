$env:BOOST_ROOT = "E:\sources\c\boost_1_71_0";
$env:Path += ";C:\python27";

$build = $args[0];

# Push-Location -Path "E:\sources\c\fivem-fork\vendor\fivem-wasm" -StackName FiveMWasm;

# cargo build --release --package cfx-component-glue

# так сказать local dev
cargo wasi build --package basic-client --release
cargo wasi build --package basic-server --release
cargo wasi build --package bench --release
cargo build --package cfx-component-glue --release

Copy-Item "E:\sources\projects\fivem-wasm\target\release\cfx_component_glue.lib" -Destination "E:\sources\c\fivem-fork\vendor\fivem-wasm\target\release\" -Force
Copy-Item "E:\sources\projects\fivem-wasm\glue\cfx-wasm-runtime.h" -Destination "E:\sources\c\fivem-fork\vendor\fivem-wasm\glue\" -Force
Copy-Item "E:\sources\projects\fivem-wasm\target\wasm32-wasi\release\basic_client.wasm" -Destination "E:\sources\c\fivem-fork\code\bin\server\windows\release\resources\main\" -Force
Copy-Item "E:\sources\projects\fivem-wasm\target\wasm32-wasi\release\basic_server.wasm" -Destination "E:\sources\c\fivem-fork\code\bin\server\windows\release\resources\main\" -Force
Copy-Item "E:\sources\projects\fivem-wasm\target\wasm32-wasi\release\bench.wasm" -Destination "E:\sources\c\fivem-fork\code\bin\server\windows\release\resources\wasmbench\" -Force

# Pop-Location -StackName FiveMWasm;

if ($build -eq "build") {
    Push-Location -Path "E:\sources\c\fivem-fork\code" -StackName FiveMWasm;

    .\tools\ci\premake5.exe vs2019 --game=server
    .\tools\ci\premake5.exe vs2019 --game=five

    Push-Location -Path "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\MSBuild\Current\Bin" -StackName FiveMWasm;
    ./MSbuild.exe "E:\sources\c\fivem-fork\code\build\server\windows\CitizenMP.sln" /t:Build /p:Configuration=Release /p:Platform=x64 -m
    ./MSbuild.exe "E:\sources\c\fivem-fork\code\build\five\CitizenMP.sln" /t:Build /p:Configuration=Release -m

    Pop-Location -StackName FiveMWasm;
    Pop-Location -StackName FiveMWasm;

    Copy-Item "E:\sources\projects\fivem-wasm\utils\components.json" -Destination "E:\sources\c\fivem-fork\code\bin\server\windows\release\" -Force;
}
