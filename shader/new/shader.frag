# version 450
# extension GL_ARB_separate_shader_objects : enable
# extension GL_ARB_shading_language_420pack : enable
# extension GL_EXT_debug_printf : enable

layout (set = 0, binding = 0) uniform Uniform {
    uint time;

	float width;
	float height;

	float rawfieldOfView;
	uint maxRayLen;

	vec2 rotation;

	uint octreeRootIndex;
	
	uint nodeAtPos;
	vec3 cameraPos;
} uniformBuffer;

struct TreeNode {
    uint mat;
	uint parent;

    uint span;
    uint spaceIndex;

	uint[8] children;
	
	float X;
	float Y;
	float Z;
};

#define HASHSCALE3 vec3(.1031, .1030, .0973)

#define detail 5
#define steps 300
#define maxdistance 30.0

//#define drawgrid
#define fog
//#define borders
#define blackborders
//#define raymarchhybrid 100
#define objects
#define emptycells 0.5
#define subdivisions 0.95 //should be higher than emptycells

#define rot(spin) mat2(cos(spin),sin(spin),-sin(spin),cos(spin))

#define sqr(a) (a*a)

//random function from https://www.shadertoy.com/view/MlsXDf
float rnd(vec4 v) { return fract(4e4*sin(dot(v,vec4(13.46,41.74,-73.36,14.24))+17.34)); }

//hash function by Dave_Hoskins https://www.shadertoy.com/view/4djSRW
vec3 hash33(vec3 p3)
{
	p3 = fract(p3 * HASHSCALE3);
    p3 += dot(p3, p3.yxz+19.19);
    return fract((p3.xxy + p3.yxx)*p3.zyx);
}

//0 is empty, 1 is subdivide and 2 is full
int getvoxel(vec3 p, float size) {
#ifdef objects
    if (p.x==0.0&&p.y==0.0) {
        return 0;
    }
#endif
    
    float val = rnd(vec4(p,size));
    
    if (val < emptycells) {
        return 0;
    } else if (val < subdivisions) {
        return 1;
    } else {
        return 2;
    }
    
    return int(val*val*3.0);
}

//ray-cube intersection, on the inside of the cube
vec3 voxel(vec3 ro, vec3 rd, vec3 ird, float size)
{
    size *= 0.5;
    
    vec3 hit = -(sign(rd)*(ro-size)-size)*ird;
    
    return hit;
}

float map(vec3 p, vec3 fp) {
    p -= 0.5;
    
    vec3 flipping = floor(hash33(fp)+0.5)*2.0-1.0;
    
    p *= flipping;
    
    vec2 q = vec2(abs(length(p.xy-0.5)-0.5),p.z);
    float len = length(q);
    q = vec2(abs(length(p.yz-vec2(-0.5,0.5))-0.5),p.x);
    len = min(len,length(q));
    q = vec2(abs(length(p.xz+0.5)-0.5),p.y);
    len = min(len,length(q));
    
    
    return len-0.1666;
}

vec3 findnormal(vec3 p, float epsilon, vec3 fp)
{
    vec2 eps = vec2(0,epsilon);
    
    vec3 normal = vec3(
        map(p+eps.yxx,fp)-map(p-eps.yxx,fp),
        map(p+eps.xyx,fp)-map(p-eps.xyx,fp),
        map(p+eps.xxy,fp)-map(p-eps.xxy,fp));
    return normalize(normal);
}

layout (location = 0) in vec2 localPos;
layout (location = 0) out vec4 fragColor;

void main() {
	vec2 curRes = vec2(uniformBuffer.width, uniformBuffer.height);
	vec2 fragCoord = gl_FragCoord.xy;
	float curTime = float(uniformBuffer.time) / 1000.0 * 0.5;
	fragColor = vec4(0.0);
    vec2 uv = (fragCoord * 2.0 - curRes) / curRes.y;
    float size = 1.0;

	
    vec3 ro = vec3(0.5+sin(curTime)*0.4,0.5+cos(curTime)*0.4,curTime);
    vec3 rd = normalize(vec3(uv,1.0));
    
	vec3 iMouse = vec3(0);
	if (length(iMouse.xy) > 40.0) {
    	rd.yz *= rot(iMouse.y/curRes.y*3.14-3.14*0.5);
    	rd.xz *= rot(iMouse.x/curRes.x*3.14*2.0-3.14);
    }
    

    vec3 lro = mod(ro,size);
    vec3 fro = ro-lro;
    vec3 ird = 1.0/max(abs(rd),0.001);
    vec3 mask;
    bool exitoct = false;
    int recursions = 0;
    float dist = 0.0;
    float fdist = 0.0;
    int i;
    float edge = 1.0;
    vec3 lastmask;
    vec3 normal = vec3(0.0);
    
    //the octree traverser loop
    //each iteration i either:
    // - check if i need to go up a level
    // - check if i need to go down a level
    // - check if i hit a cube
    // - go one step forward if octree cell is empty
    // - repeat if i did not hit a cube
    for (i = 0; i < steps; i++)
    {
        if (dist > maxdistance) break;
        
        //i go up a level
        if (exitoct)
        {
            
            vec3 newfro = floor(fro/(size*2.0))*(size*2.0);
            
            lro += fro-newfro;
            fro = newfro;
            
            recursions--;
            size *= 2.0;
            
            exitoct = (recursions > 0) && (abs(dot(mod(fro/size+0.5,2.0)-1.0+mask*sign(rd)*0.5,mask))<0.1);
        }
        else
        {
            //checking what type of cell it is: empty, full or subdivide
            int voxelstate = getvoxel(fro,size);
            if (voxelstate == 1 && recursions > detail)
            {
                voxelstate = 0;
            }
            
            if(voxelstate == 1&&recursions<=detail)
            {
                //if(recursions>detail) break;

                recursions++;
                size *= 0.5;

                //find which of the 8 voxels i will enter
                vec3 mask2 = step(vec3(size),lro);
                fro += mask2*size;
                lro -= mask2*size;
            }
            //move forward
            else if (voxelstate == 0||voxelstate == 2)
            {
                //raycast and find distance to nearest voxel surface in ray direction
                //i don't need to use voxel() every time, but i do anyway
                vec3 hit = voxel(lro, rd, ird, size);

                mask = vec3(lessThan(hit,min(hit.yzx,hit.zxy)));
                float len = dot(hit,mask);

				if (voxelstate == 2) {
                    break;
                }

                //moving forward in ray direction, and checking if i need to go up a level
                dist += len;
                fdist += len;
                lro += rd*len-mask*sign(rd)*size;
                vec3 newfro = fro+mask*sign(rd)*size;
                exitoct = (floor(newfro/size*0.5+0.25)!=floor(fro/size*0.5+0.25))&&(recursions>0);
                fro = newfro;
                lastmask = mask;
            }
        }
    }
    ro += rd*dist;
    if(i < steps && dist < maxdistance)
    {
    	float val = fract(dot(fro,vec3(15.23,754.345,3.454)));
        vec3 color = sin(val*vec3(39.896,57.3225,48.25))*0.5+0.5;
    	fragColor = vec4(color*(normal*0.25+0.75),1.0);
        fragColor = 1.0-(1.0-fragColor)*edge;
    } else {
        fragColor = vec4(1.0-edge);
    }
    fragColor = sqrt(fragColor);
}
