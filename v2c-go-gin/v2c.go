package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/goccy/go-yaml"
	"io/ioutil"
	"net/http"
	"strings"
)

func main() {
	// resyaml, err := translate("https://yukari.kelu.org/v2/04669d2c04ff47d985673a4e5b0c7ea1")
	// if err != nil {
	// 	fmt.Printf("%s %q\n", resyaml, err)
	// 	return
	// }
	// fmt.Printf("%s\n", resyaml)

	r := gin.Default()
	r.GET("/v2rayn_to_clash", func(c *gin.Context) {
		upstreamUrl := c.Query("from")
		resyaml, err := translate(upstreamUrl)
		if err != nil {
			v := struct {
				progress string
				errmsg   error
			}{resyaml, err}
            bytes, err := yaml.Marshal(v)
			if err != nil {
				c.String(http.StatusInternalServerError, fmt.Sprintf("progress: FORMAT_ERR, errmsg: %q", err))
			} else {
				c.String(http.StatusInternalServerError, string(bytes))
			}
		} else {
			c.String(http.StatusOK, resyaml)
		}
	})
	r.Run(":8423")
}

type V2rayN struct {
	Address      string `json:"add"`
	Port         string `json:"port"`
	Uuid         string `json:"id"`
	AlterId      string `json:"aid"`
	Network      string `json:"net"`
	FriendlyName string `json:"ps"`
	Type         string `json:"type"`
	FakeHost     string `json:"host"`
	Path         string `json:"path"`
	Tls          string `json:"tls"`
}

type Clash struct {
	FriendlyName  string `yaml:"name"`
	Type          string `yaml:"type"`
	Address       string `yaml:"server"`
	Port          string `yaml:"port"`
	Uuid          string `yaml:"uuid"`
	AlterId       string `yaml:"alterId"`
	Cipher        string `yaml:"cipher"`
	UsingFakeHost bool   `yaml:"skip-cert-verify"`
	Network       string `yaml:"network"`
	Path          string `yaml:"ws-path"`
	Headers       struct {
		Host string `yaml:"host"`
	} `yaml:"ws-headers"`
	Tls bool `yaml:"tls,omitempty"`
}

func translate(url string) (string, error) {
	resp, err := http.Get(url)
	if err != nil {
		return fmt.Sprintf("UPSTREAM_GET_ERROR(%s)", url), err
	}
	defer resp.Body.Close()
	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return fmt.Sprintf("UPSTREAM_READ_ERROR"), err
	}

	donce, err := base64.StdEncoding.DecodeString(string(body))
	if err != nil {
		return fmt.Sprintf("BASE64_DECODE_1(%s)", string(body)), err
	}

	dosplits := strings.Split(string(donce), "\n")

	var clashps []Clash

	for _, dosplit := range dosplits {
		if strings.HasPrefix(dosplit, "vmess://") {
			dtwice, err := base64.StdEncoding.DecodeString(dosplit[8:])
			if err != nil {
				return fmt.Sprintf("BASE64_DECODE_2(%s)", dosplit), err
			}

			var v2rayn V2rayN
			err = json.Unmarshal(dtwice, &v2rayn)
			if err != nil {
                return fmt.Sprintf("JSON_PARSE(%s)", dtwice), err
			}

			clash := Clash{}

			clash.FriendlyName = v2rayn.FriendlyName
			clash.Type = "vmess"
			clash.Address = v2rayn.Address
			clash.Port = v2rayn.Port
			clash.Uuid = v2rayn.Uuid
			clash.AlterId = v2rayn.AlterId
			clash.Cipher = "auto"
			clash.UsingFakeHost = true
			clash.Network = v2rayn.Network
			clash.Path = v2rayn.Path
			clash.Headers.Host = v2rayn.FakeHost
			clash.Tls = v2rayn.Tls == "tls"

			clashps = append(clashps, clash)
		}
	}

	var clasho struct {
		Proxies []Clash `yaml:"proxies"`
	}
	clasho.Proxies = clashps

	resyaml, err := yaml.Marshal(clasho)
	if err != nil {
		return fmt.Sprintf("YAML_ENCODE(%v)", clasho), err
	}
	return string(resyaml), nil
}
