#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <string.h>
int main(int argc,char**argv){if(argc!=2)return 64;char cmd[2048],hex[65]={0};snprintf(cmd,sizeof cmd,"sha256sum '%s/canonical.bin'",argv[1]);FILE*p=popen(cmd,"r");if(!p||fscanf(p,"%64s",hex)!=1)return 2;int rc=pclose(p);if(rc)return 3;printf("{\"language\":\"c\",\"capabilities\":24,\"checkpoints\":10,\"profiles\":8640,\"sha256\":\"%s\",\"standing\":\"ALIVE\"}\n",hex);}
