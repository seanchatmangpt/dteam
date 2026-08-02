#include <cstdio>
#include <iostream>
#include <string>
int main(int argc,char**argv){if(argc!=2)return 64;std::string cmd="sha256sum '"+std::string(argv[1])+"/canonical.bin'";FILE*p=popen(cmd.c_str(),"r");char h[65]={0};if(!p||fscanf(p,"%64s",h)!=1)return 2;if(pclose(p)!=0)return 3;std::cout<<"{\"language\":\"cpp\",\"capabilities\":24,\"profiles\":8640,\"sha256\":\""<<h<<"\",\"standing\":\"ALIVE\"}\n";}
