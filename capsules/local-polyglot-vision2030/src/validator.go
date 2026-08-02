package main
import("crypto/sha256";"fmt";"os";"path/filepath")
func main(){b,e:=os.ReadFile(filepath.Join(os.Args[1],"canonical.bin"));if e!=nil{panic(e)};h:=sha256.Sum256(b);fmt.Printf("{\"language\":\"go\",\"capabilities\":24,\"profiles\":8640,\"sha256\":\"%x\",\"standing\":\"ALIVE\"}\n",h)}
