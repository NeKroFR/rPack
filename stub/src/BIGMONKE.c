/*
Obfuscate using Tigress: https://tigress.wtf/index.html
---

$ tigress  --Environment=x86_64:Linux:Gcc:4.6 \
    --Transform=InitBranchFuns --InitBranchFunsCount=1 \
    --Transform=InitEntropy \
    --Transform=InitImplicitFlow --Functions=main \
    --Transform=InitOpaque --Functions=main  --InitOpaqueStructs=list,array,env \
    --Transform=Virtualize --Functions=main \
         --VirtualizeAddOpaqueToVPC=true --VirtualizeImplicitFlowPC=PCInit, PCUpdate \
         BIGMONKE.c --out=TigerMONKE.c && gcc TigerMONKE.c -static -s -o BIGMONKE
*/

#include <stdio.h>
#include <sys/prctl.h>
// Tigress dependencies
#include<time.h>
#include<stdlib.h>

int main() {
    prctl(PR_SET_DUMPABLE, 0);
    const char bigMonke[] = 
        "            ,.-\" \"-.,\n"
        "           /   ===   \\\n"
        "          /  =======  \\\n"
        "       __|  (o)   (0)  |__\n"
        "      / _|    .---.    |_ \\\n"
        "     | /.----/ O O \\----.\\ |\n"
        "      \\/     |     |     \\/\n"
        "      |                   |\n"
        "      |                   |\n"
        "      |                   |\n"
        "      _\\   -.,_____,.-   /_\n"
        "  ,.-\"  \"-.,_________,.-\"  \"-.,\n"
        " /         |   BIG   |         \\\n"
        "|          l. MONKE .l          |\n";
    printf("%s", bigMonke);
    int *p = NULL;
    *p = 0;
    printf("%d", *p);
    return 0;
}
