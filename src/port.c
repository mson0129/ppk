/* PPK 50-stage static source port. The generated routines are native C calls
 * compiled ahead of time; no EXE, CPU decoder, DOS or VGA device is run. */
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#ifdef NATIVE_TEST
#include <stdio.h>
#endif

#include "assets.generated.h"
#define DATA 0xf0a0u
#define BG 0x20000u
#define WIDTH 320
#define HEIGHT 200
#define API __attribute__((visibility("default")))
static uint8_t mem[0x50000];
static uint8_t screen[WIDTH*HEIGHT];
static uint32_t rgba[WIDTH*HEIGHT];
static uint16_t ax,bx,cx,dx,si,di,bp,sp,ds,es,ss,cs;
static int cf,zf,sf,of;
static uint32_t error_code;
static uint32_t audio_event_id;
static uint32_t finished_run;
static int intro_x, intro_previous_x, intro_direction;
enum { PLAYING, DYING, CONTINUE_SCREEN, INTRO, STAGE_CLEAR };
#define STAGE_COUNT (sizeof asset_stage_ppk / 330)
static int phase, wait_ticks, native_ticks, muted, advance_ready, title_space_release;
static int tone_hz, tone_ticks, song_index, song_count;
static uint8_t song[128];
static uint32_t colors[16];
static int previous_x[64], previous_y[64], previous_count;
static void capture_positions(void);

static uint16_t pit_divisor; static int pit_low, pit_enabled;
static uint8_t rd8(uint32_t a) { if(a>=sizeof mem){error_code=a; return 0;} return mem[a]; }
static uint16_t rd16(uint32_t a){return rd8(a)|(rd8(a+1)<<8);}
static void wr8(uint32_t a,uint32_t v){if(a>=sizeof mem){error_code=a;return;}mem[a]=v;}
static void wr16(uint32_t a,uint32_t v){wr8(a,v);wr8(a+1,v>>8);}
static uint16_t state(int o){return rd16(DATA+o);}
static void set(int o,int v){wr16(DATA+o,v);}
static void push16(uint16_t v){sp-=2;wr16((ss<<4)+sp,v);}
static uint16_t pop16(void){uint16_t v=rd16((ss<<4)+sp);sp+=2;return v;}
static void fail(uint32_t code){error_code=code;}
static void flags_logic(uint32_t a,uint32_t b,uint32_t t,int bits){uint32_t mask=(1u<<bits)-1;zf=(t&mask)==0;sf=(t>>(bits-1))&1;cf=of=0;}
static void flags_add(uint32_t a,uint32_t b,uint32_t t,int bits){uint32_t mask=(1u<<bits)-1;zf=(t&mask)==0;sf=(t>>(bits-1))&1;cf=(t>>bits)&1;of=((~(a^b)&(a^t))>>(bits-1))&1;}
static void flags_sub(uint32_t a,uint32_t b,uint32_t t,int bits){uint32_t mask=(1u<<bits)-1;zf=(t&mask)==0;sf=(t>>(bits-1))&1;cf=(t>>bits)&1;of=(((a^b)&(a^t))>>(bits-1))&1;}
static uint32_t shift_shl(uint32_t a,int n,int bits){n&=31;if(!n)return a;uint32_t t=a<<n;int oldsf=(a>>(bits-1))&1;flags_logic(0,0,t,bits);cf=(n<=bits)?((a>>(bits-n))&1):0;of=sf^cf;return t;}
static uint32_t shift_shr(uint32_t a,int n,int bits){n&=31;if(!n)return a;uint32_t t=a>>n;flags_logic(0,0,t,bits);cf=(n<=bits)?((a>>(n-1))&1):0;of=(a>>(bits-1))&1;return t;}
static uint32_t shift_sar(uint32_t a,int n,int bits){n&=31;if(!n)return a;int32_t s=bits==8?(int8_t)a:(int16_t)a;uint32_t t=s>>n;flags_logic(0,0,t,bits);cf=(a>>(n-1))&1;of=0;return t;}
static void sdiv32(int32_t v,int16_t d){if(!d){fail(1);return;}ax=v/d;dx=v%d;}
static void speaker_port(uint16_t port,uint16_t v){
 if(port==0x42){if(!pit_low){pit_divisor=(pit_divisor&0xff00)|(v&255);pit_low=1;}else{pit_divisor=(pit_divisor&255)|((v&255)<<8);pit_low=0;}}
 if(port==0x61){pit_enabled=(v&3)==3;if(pit_enabled&&pit_divisor)tone_hz=1193182/pit_divisor;}
}
/* Original assets use four 1-bit color planes; this is a data decoder, not VGA I/O. */
static int source_pixel(uint32_t source,int x,int y,int w,int h){int color=0;for(int p=0;p<4;p++)color|=((rd8(source+p*(w/8)*h+y*(w/8)+x/8)>>(7-(x&7)))&1)<<p;return color;}
static void bg_pixel(int x,int y,int c){if(x<0||x>=320||y<0||y>=200)return;for(int p=0;p<4;p++){uint32_t a=BG+p*8000+y*40+x/8;uint8_t bit=0x80>>(x&7);mem[a]=(mem[a]&~bit)|((c&(1<<p))?bit:0);}}
static void draw_tile(int x,int y,uint32_t src,int height){x=(x>>3)<<3;for(int row=0;row<height;row++)for(int col=0;col<16;col++)bg_pixel(x+col,y+row,source_pixel(src,col,row,16,height));}
static void copy_background(uint16_t seg){uint32_t source=(uint32_t)seg<<4;for(int y=0;y<200;y++)for(int x=0;x<320;x++){int c=0;for(int p=0;p<4;p++)c|=((rd8(source+p*8000+y*40+x/8)>>(7-(x&7)))&1)<<p;screen[y*320+x]=c;}}
static void draw_sprite(int x,int y,uint32_t src,int h,int op){for(int row=0;row<h;row++)for(int col=0;col<16;col++){int xx=x+col,yy=y+row;if(xx<0||xx>=320||yy<0||yy>=200)continue;int c=source_pixel(src,col,row,16,h);uint8_t*dst=&screen[yy*320+xx];if(op==1)*dst&=c;else if(op==2)*dst|=c;else if(op==3)*dst^=c;else *dst=c;}}
static void draw_text(int x,int y,int color,uint32_t text){for(int k=0;k<128;k++){uint8_t ch=rd8(text+k);if(!ch)break;uint32_t font=DATA-110+ch*8;for(int row=0;row<8;row++)for(int col=0;col<8;col++){int bit=(rd8(font+row)>>(7-col))&1;/* Original 1-bit font: zero in selected color planes, glyph in others. */bg_pixel(((x+k*8)>>3)*8+col,y+row,bit?(15^color):0);}}}
static void decimal(char *out, uint32_t n, int width, char pad){char tmp[16];int k=0;do{tmp[k++]=(char)('0'+n%10);n/=10;}while(n);int p=0;while(p<width-k)out[p++]=pad;while(k)out[p++]=tmp[--k];out[p]=0;}
static void format_value(char*out,const char*fmt,uint32_t n){int p=0;for(int j=0;fmt[j]&&p<80;j++){if(fmt[j]!='%'){out[p++]=fmt[j];continue;}j++;char pad=' ';if(fmt[j]=='0'){pad='0';j++;}int width=0;while(fmt[j]>='0'&&fmt[j]<='9')width=width*10+fmt[j++]-'0';if(fmt[j]=='l')j++;char num[24];decimal(num,n,width,pad);for(int k=0;num[k]&&p<80;k++)out[p++]=num[k];}out[p]=0;}
static uint16_t arg(int n){return rd16((ss<<4)+(uint16_t)(sp+n*2));}
static void platform_call(uint16_t seg,uint16_t ip){
 if(seg==0x2fe){
  switch(ip){
   case 0xb: case 0x22: case 0x46: return; /* browser owns timing and display */
   case 0x5f:copy_background(arg(0));return;
   case 0x9d:draw_tile(arg(0),arg(1),((uint32_t)arg(3)<<4)+arg(2),arg(4));return;
   case 0x179:draw_sprite(arg(0),arg(1),((uint32_t)arg(3)<<4)+arg(2),arg(4),arg(5)&255);return;
   case 0x23b:draw_text(arg(0),arg(1),arg(2)&255,((uint32_t)arg(4)<<4)+arg(3));return;
  }
 }
 if(seg==0){
  switch(ip){
   case 0x179f:{int v=(int16_t)arg(0);ax=v<0?-v:v;}return;
   case 0x2bd3:{uint32_t a=((uint32_t)dx<<16)|ax,b=((uint32_t)cx<<16)|bx,t=a*b;ax=t;dx=t>>16;return;}
   case 0x2bf4:{uint32_t a=((uint32_t)arg(1)<<16)|arg(0),b=((uint32_t)arg(3)<<16)|arg(2);if(!b){fail(2);return;}uint32_t q=a/b;ax=q;dx=q>>16;sp+=8;return;}
   case 0x14c7:{uint32_t dst=((uint32_t)arg(1)<<4)+arg(0),fmt=((uint32_t)arg(3)<<4)+arg(2);char out[96];const char*f=(char*)mem+fmt;int v=(int16_t)arg(4);if(strstr(f,"lu")){uint32_t n=((uint32_t)arg(5)<<16)|arg(4);format_value(out,f,n);}else format_value(out,f,v);for(int i=0;i<96;i++){wr8(dst+i,out[i]);if(!out[i])break;}return;}
  }
 }
 fail(((uint32_t)seg<<16)|ip);
}
static void replace_game(int ip){
 if(ip==0xc6){song_count=rd16((ss<<4)+(uint16_t)(sp+8));if(song_count>128)song_count=128;uint32_t src=((uint32_t)rd16((ss<<4)+(uint16_t)(sp+6))<<4)+rd16((ss<<4)+(uint16_t)(sp+4));for(int k=0;k<song_count;k++)song[k]=rd8(src+k);song_index=0;for(int k=0;k<song_count;k++)wr8(DATA+0x48e + k,song[k]);set(0x1598,0);set(0x1594,song_count);set(0x158a,1);return;}
 if(ip==0x19b||ip==0x1d1||ip==0x2e6)return; /* actual palette is applied at presentation */
 fail(ip);
}
#include "game.generated.c"
static void invoke(void(*f)(void)){push16(cs);push16(0);f();}
static void present(void){for(int i=0;i<64000;i++)rgba[i]=colors[screen[i]&15];}
static void game_palette(void){
 for(int i=0;i<16;i++){int c=rd8(DATA+0x56d + i);colors[i]=0xff000000u|((uint32_t)(asset_stage_pp3[c*3+2]*255/63)<<16)|((uint32_t)(asset_stage_pp3[c*3+1]*255/63)<<8)|(asset_stage_pp3[c*3]*255/63);}
}
static void load_stage(void){
 int stage=state(0x13f4);if(stage<1||stage>STAGE_COUNT){fail(0x1eeb);return;}
 game_palette();
 push16(stage);invoke(game_1eeb);sp+=2;
 /* Original scenery selection: stage modulo 27, including the zero slot. */
 memcpy(mem+0x9220 + 0x700,mem+0xa220 + (stage%27)*128,128);
 invoke(game_260d);invoke(game_1f34);invoke(game_09e9);capture_positions();present();
}
API void init(void){
 memset(mem,0,sizeof mem);memset(screen,0,sizeof screen);memset(rgba,0,sizeof rgba);
 memcpy(mem+DATA,asset_initial_data_bin,sizeof asset_initial_data_bin);
 for(int i=0;i<16;i++){int c=rd8(DATA+0x56d + i);colors[i]=0xff000000u|((uint32_t)(asset_stage_pp3[c*3+2]*255/63)<<16)|((uint32_t)(asset_stage_pp3[c*3+1]*255/63)<<8)|(asset_stage_pp3[c*3]*255/63);}

 memcpy(mem+0x6020,asset_ppk_b16,48*128);memcpy(mem+0x7920,asset_ppk_b16+48*128,49*128);memcpy(mem+0x9220,asset_ppk_b16+97*128,31*128);memcpy(mem+0xa220,asset_ppk_b16+128*128,28*128);
 memcpy(mem+DATA+0x15f0,asset_stage_pp4,sizeof asset_stage_pp4);
 memcpy(mem+0xb020,asset_stage_ppk,sizeof asset_stage_ppk);
 ax=bx=cx=dx=si=di=bp=0;ds=0xf0a;ss=0x4000;es=0;cs=0x327;sp=0xfffe;
 cf=zf=sf=of=0;error_code=0;phase=0;advance_ready=0;title_space_release=0;wait_ticks=0;native_ticks=0;audio_event_id=0;finished_run=0;tone_hz=0;tone_ticks=0;song_count=0;song_index=0;pit_low=0;pit_enabled=0;
 set(0x13f4,1);set(0x159c,BG>>4);set(0x1398,5);set(0x159a,5);set(0x158c,10000);set(0x1212,1);set(0xcb6,0);set(0xcb8,0);set(0x561,248);set(0x563,168);load_stage();
}
/* Original 0327:2477 screen loader: 128-byte header, PCX-style RLE,
 * scanline planes in the unusual order 3,0,1,2. */
static void load_screen(const uint8_t *file,size_t size){
 size_t pos=128;int written=0;
 while(written<32000&&pos<size){int value=file[pos++],count=1;if((value&0xc0)==0xc0){count=value&63;if(pos>=size){fail(0x2477);return;}value=file[pos++];}
  while(count--&&written<32000){int row=written/160,plane=((written/40)%4+3)%4,col=written%40;mem[BG+plane*8000+row*40+col]=value;written++;}
 }
 if(written!=32000){fail(0x2477);return;}
 for(int i=0;i<16;i++){int c=rd8(DATA+0x56d+i);colors[i]=0xff000000u|((uint32_t)((file[16+c*3+2]>>2)*255/63)<<16)|((uint32_t)((file[16+c*3+1]>>2)*255/63)<<8)|((file[16+c*3]>>2)*255/63);}
}
API void show_intro(void){
 load_screen(asset_stage_pp1,sizeof asset_stage_pp1);
 intro_x=intro_previous_x=186;intro_direction=2;phase=3;native_ticks=0;
 song_count=54;song_index=0;for(int i=0;i<song_count;i++)song[i]=rd8(DATA+0x511+i);set(0x158a,1);set(0x1598,0);set(0x1594,song_count);
 copy_background(BG>>4);draw_sprite(intro_x,119,0x7920+0x1800,16,1);draw_sprite(intro_x+30,113,0x7920+0x1800,16,1);present();
}
API void start_stage(int stage){
 if(stage<1||stage>STAGE_COUNT)return;
 init();if(stage!=1){set(0x13f4,stage);load_stage();}
}
static void show_continue(void){
 load_screen(asset_stage_pp2,sizeof asset_stage_pp2);
 char text[32];format_value(text,(char*)mem+DATA+0x7fb,state(0x159a));
 memcpy(mem+DATA+0xc44,text,strlen(text)+1);draw_text(240,178,2,DATA+0xc44);
 copy_background(BG>>4);present();phase=CONTINUE_SCREEN;advance_ready=0;
}
API uint32_t get_stage(void){return state(0x13f4);}
API uint32_t stage_count(void){return STAGE_COUNT;}
API void key(int scan,int down){
 /* Consume the title's Space until every input source releases it. */
 if(scan==57&&title_space_release){if(!down)title_space_release=0;down=0;}
 if(scan>=0&&scan<128)wr8(DATA+0x10ae + scan,down?1:0);
}
API void release_keys(void){memset(mem+DATA+0x10ae,0,128);}
API void tick(void){
 if(error_code)return; capture_positions(); native_ticks++;
 if(song_index<song_count){int note=song[song_index++];set(0x1598,song_index);int divisor=state(0x3c6 + note*2);tone_hz=divisor?1193182/divisor:0;set(0x158a,1);}else{tone_hz=0;set(0x158a,0);}
 if(phase==INTRO){if(rd8(DATA+0x10e7)){int stage=state(0x13f4);start_stage(stage);title_space_release=1;return;}intro_previous_x=intro_x;intro_x+=intro_direction;if(intro_x>=213||intro_x<=184)intro_direction=-intro_direction;copy_background(BG>>4);draw_sprite(intro_x,119,0x7920+0x1800,16,1);draw_sprite(intro_x+30,113,0x7920+0x1800,16,1);present();return;}
 if(phase==STAGE_CLEAR||phase==CONTINUE_SCREEN){
  int pressed=rd8(DATA+0x10e7)||rd8(DATA+0x10ca);
  if(!pressed)advance_ready=1;
  if(pressed&&advance_ready){
   if(phase==STAGE_CLEAR){set(0x13f4,state(0x13f4)%STAGE_COUNT+1);}
   else if(state(0x159a)>0){set(0x159a,state(0x159a)-1);set(0x1398,5);}
   else {init();show_intro();return;}
   set(0x561,248);set(0x563,168);release_keys();load_stage();phase=PLAYING;advance_ready=0;
  }
  return;
 }
 if(phase==1){if(wait_ticks>0){wait_ticks--;return;}set(0x561,248);set(0x563,168);set(0x1398,state(0x1398)-1);if((int16_t)state(0x1398)<0){show_continue();return;}load_stage();phase=0;return;}
 invoke(game_11c1);invoke(game_12f5);invoke(game_1429);invoke(game_0d98);invoke(game_1cca);invoke(game_1db1);invoke(game_1e09);invoke(game_15a3);
 /* Completion is latched at the last-candy update, before rendering/bonus UI. */
 if(state(0x13f4)==STAGE_COUNT&&state(0x13fa)==0)finished_run=1;
 invoke(game_09e9);
 if(state(0x583)){tone_hz=1193182/(state(0x139a)?state(0x139a):1);set(0x139a,state(0x139a)+10);}
 if(state(0x585)){phase=1;wait_ticks=12;audio_event_id++;}
 if(state(0x13fa)==0&&phase==PLAYING){phase=STAGE_CLEAR;advance_ready=0;invoke(game_1db1);invoke(game_0d1e);draw_text(80,100,5,DATA+0x849);char bonus[16];decimal(bonus,(135-state(0x1396))*20,0,' ');memcpy(mem+DATA+0xc44,bonus,strlen(bonus)+1);draw_text(192,100,2,DATA+0xc44);copy_background(BG>>4);}
 present();
}
/* Rendering interpolates positions only. All collision/game state stays on the
 * original fixed clock, and a render call cannot advance the game. */
typedef struct {int x,y,sprite,visible;} Actor;
static int actors(Actor*out){int n=0;
 for(int i=0;i<state(0x13f0)&&n<63;i++){int a=0x147e + i*8;out[n++]=(Actor){state(a),state(a+2),state(a+6),1};}
 for(int i=0;i<state(0x1592)&&n<63;i++){int a=0x15a0 + i*8;out[n++]=(Actor){state(a),state(a+2),state(a+6),1};}
 for(int i=0;i<state(0x13fc)&&n<63;i++){int a=0x14fe + i*14;out[n++]=(Actor){state(a),state(a+2),state(a+6),state(a+8)&&state(a+10)%2};}
 if(n<63)out[n++]=(Actor){state(0x561),state(0x563),state(0x567),1};
 for(int i=0;i<8&&n<64;i++){int a=0x13a0 + i*8;out[n++]=(Actor){state(a),state(a+2),state(a+4),state(a+6)};}
 return n;
}
static void capture_positions(void){Actor a[64];previous_count=actors(a);for(int i=0;i<previous_count;i++){previous_x[i]=a[i].x;previous_y[i]=a[i].y;}}
API void render_interpolated(int fraction){
 if(phase==3){int x=intro_previous_x+(intro_x-intro_previous_x)*fraction/1000;copy_background(BG>>4);draw_sprite(x,119,0x7920+0x1800,16,1);draw_sprite(x+30,113,0x7920+0x1800,16,1);present();return;}
 if(phase!=0){present();return;}
 if(fraction<0)fraction=0;if(fraction>1000)fraction=1000;
 copy_background(BG>>4);Actor a[64];int n=actors(a);
 for(int i=0;i<n;i++){if(!a[i].visible)continue;int x=a[i].x,y=a[i].y;
  if(i<previous_count){int vx=x-previous_x[i],vy=y-previous_y[i];if(vx>=-8&&vx<=8&&vy>=-8&&vy<=8){x=previous_x[i]+(vx*fraction+(vx>=0?500:-500))/1000;y=previous_y[i]+(vy*fraction+(vy>=0?500:-500))/1000;}}
  draw_sprite(x,y,0x7920 + a[i].sprite*128,16,1);
  draw_sprite(x,y,0x6020 + a[i].sprite*128,16,3);
 }
 present();
}
API uint32_t framebuffer(void){return (uint32_t)(uintptr_t)rgba;}
API uint32_t audio_event(void){return audio_event_id;}
API uint32_t audio_frequency(void){return muted?0:tone_hz;}
API void mute(int m){muted=m;}
API uint32_t debug_state(int offset){return state(offset);}
API uint32_t run_finished(void){return finished_run;}
API uint32_t get_phase(void){return phase;}
API uint32_t get_error(void){return error_code;}
API uint32_t data_pointer(void){return (uint32_t)(uintptr_t)mem;}
API uint32_t frame_count(void){return native_ticks;}
/* Tick-boundary digest excludes presentation buffers: rendering is read-only. */
API uint32_t replay_hash(void){
 uint32_t hash=2166136261u;
 for(size_t i=0;i<sizeof mem;i++)hash=(hash^mem[i])*16777619u;
 const uint32_t extra[]={phase,wait_ticks,native_ticks,advance_ready,title_space_release,finished_run,intro_x,intro_direction};
 for(size_t i=0;i<sizeof extra/sizeof extra[0];i++)hash=(hash^extra[i])*16777619u;
 return hash;
}
#ifdef NATIVE_TEST
int main(void){init();for(int i=0;i<600;i++){if(i==1)key(75,1);if(i==100)key(75,0);tick();if(error_code){printf("ERROR %x tick %d\n",error_code,i);return 1;}}printf("x=%d y=%d fruits=%d score=%d sp=%x phase=%d\n",state(0x561),state(0x563),state(0x13fa),state(0xcb6),sp,phase);FILE*f=fopen("/private/tmp/ppk-native.rgba","wb");fwrite(rgba,sizeof rgba,1,f);fclose(f);return 0;}
#endif
#ifdef TEST_ORACLE
API void *test_memory(void){return mem;}
API void test_routine(int address){switch(address){
case 0x4db:invoke(game_04db);break;case 0xd98:invoke(game_0d98);break;
case 0x11c1:invoke(game_11c1);break;case 0x12f5:invoke(game_12f5);break;
case 0x1429:invoke(game_1429);break;case 0x15a3:invoke(game_15a3);break;
case 0x1cca:invoke(game_1cca);break;case 0x1db1:invoke(game_1db1);break;
case 0x1e09:invoke(game_1e09);break;case 0x1f34:invoke(game_1f34);break;
case 0x260d:invoke(game_260d);break;default:fail(address);break;}}
#endif
